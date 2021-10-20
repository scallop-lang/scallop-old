use scallop_runtime::{wmc::*, tags::*};
use anyhow::Result;
use tch::{nn, nn::ModuleT, nn::OptimizerConfig, Device, Tensor, index::*, Kind, Reduction};
use profiler::Profiler;
use mnist_exp::*;
use structopt::StructOpt;
// use rayon::prelude::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "count-correct-3")]
struct Options {
  #[structopt(short = "g", long = "gpu")]
  cuda_gpu: Option<usize>,

  #[structopt(short, long, default_value = "3")]
  k: usize,

  #[structopt(short = "b", long = "batch-size", default_value = "10")]
  batch_size: i64,

  #[structopt(short = "l", long = "learning-rate", default_value = "0.001")]
  learning_rate: f64,

  #[structopt(short = "e", long = "epochs", default_value = "1")]
  num_epochs: usize,

  #[structopt(long = "log-dir", default_value = "log")]
  log_dir: String,

  #[structopt(short = "s", long, default_value = "1234")]
  seed: i64,
}

fn run<const K: usize>(opt: &Options) -> Result<()> {
  let log_dir = &opt.log_dir;
  let batch_size = opt.batch_size;
  let num_epochs = opt.num_epochs;
  let test_tasks = 1000;
  let num_possible_digits = 10;
  let num_digits_per_task = 3;
  let num_facts_per_task = 30;
  let out_dim = 4;
  let learning_rate = opt.learning_rate;
  tch::manual_seed(opt.seed);

  let m = tch::vision::mnist::load_dir("data")?;
  let num_images = m.train_images.size()[0];
  let num_tasks = num_images / num_digits_per_task;

  let file_name = format!("{}/cc_2_k_{}_lr_{}_bs_{}_rs_{}.log", log_dir, K, learning_rate, batch_size, opt.seed);
  let mut profiler = Profiler::new(&file_name);
  profiler.start("total_time");

  let dev = match opt.cuda_gpu {
    Some(g) => Device::Cuda(g),
    None => Device::cuda_if_available(),
  };
  let vs = nn::VarStore::new(dev);
  let net = MnistNet::new(&vs.root());
  let mut opt = nn::Adam::default().build(&vs, learning_rate)?;
  let wmc = DiffTopKProbProofsWMC2::<K>;

  for epoch in 1..=num_epochs {
    let mut ith_batch = 0;
    let mut ith_task = 0;

    let epoch_name = format!("epoch_{}", epoch);
    profiler.start(&epoch_name);

    for (bimages, blabels) in m.train_iter(batch_size * num_digits_per_task).shuffle().to_device(vs.device()) {
      profiler.start("batch_time");
      opt.zero_grad();
      profiler.start("prediction");

      // Make prediction on all the digit images in the batch
      let raw_y_preds = net.forward_t(&bimages, true);

      profiler.end("prediction");

      profiler.start("datalog_prepare");

      let y_pred_shape = raw_y_preds.size();
      let num_options = y_pred_shape[y_pred_shape.len() - 1];

      // Loop through each pair of images & their predictions
      let batch_num = y_pred_shape[0] / num_digits_per_task;

      // First pass, synchronously generate all the labels
      let (data, y_gt_arr) : (Vec<_>, Vec<_>) = (0..batch_num).map(|i| {

        // Get the data
        let first = raw_y_preds.i((i * num_digits_per_task, ..));
        let second = raw_y_preds.i((i * num_digits_per_task + 1, ..));
        let third = raw_y_preds.i((i * num_digits_per_task + 2, ..));

        let first_y : i64 = blabels.i(i * num_digits_per_task).into();
        let second_y : i64 = blabels.i(i * num_digits_per_task + 1).into();
        let third_y : i64 = blabels.i(i * num_digits_per_task + 2).into();

        // Compute ground truth
        let y = vs.root().var_copy("y", &Tensor::of_slice(
          &(0..out_dim)
            .map(|i| {
              match i {
                0 => 0.0, // 0 correct
                1 => 0.333, // 1 correct
                2 => 0.666, // 2 correct
                3 => 1.0, // all correct
                _ => panic!("Not possible"),
              }
            })
            .collect::<Vec<_>>()
        ).set_requires_grad(false)).set_requires_grad(false);

        ((first_y, second_y, third_y, first, second, third), y)
      }).unzip();

      profiler.end("datalog_prepare");

      profiler.start("datalog_execute");

      // Execute all the datalog programs and get their contexts + elements
      let tasks = data.into_iter().map(|(first_y, second_y, third_y, first, second, third)| {
        let mut prog = CountCorrect3::<DiffTopKProbProofs<K>>::new();
        prog.digit().insert_diff_disjunction(
          to_digit_disjunction(&first, 0, num_options)
        );
        prog.digit().insert_diff_disjunction(
          to_digit_disjunction(&second, 1, num_options)
        );
        prog.digit().insert_diff_disjunction(
          to_digit_disjunction(&third, 2, num_options)
        );
        prog.gt().insert_one_ground((first_y, second_y, third_y));

        prog.run();

        let results = prog.cc_3().complete().into_iter().into_iter().collect::<Vec<_>>();

        (prog.semiring_context().clone(), results)
      }).collect::<Vec<_>>();

      profiler.end("datalog_execute");

      profiler.start("wmc");

      // Perform parallel wmc
      let results = (0..tasks.len() * out_dim as usize).map(|i| {
        let task_id = i / out_dim as usize;
        let elem_id = i % out_dim as usize;
        let ctx = &tasks[task_id].0;
        let elem = &tasks[task_id].1[elem_id];
        let pred_num = &elem.tup;
        let tag = &elem.tag;
        let (prob, derivs) = wmc.wmc(ctx, tag);
        (pred_num, prob, derivs)
      }).collect::<Vec<_>>();

      profiler.end("wmc");

      profiler.start("loss");

      let (y_preds, losses): (Vec<_>, Vec<_>) = (0..tasks.len()).map(|i| {
        let mut probs = vec![0.0; out_dim as usize];
        for j in 0..out_dim as usize {
          let index = i * out_dim as usize + j;
          let result = &results[index];
          probs[*result.0 as usize] = result.1;
        }

        let y_pred = vs.root().var_copy("y_p", &Tensor::of_slice(&probs)).set_requires_grad(true);
        let y_gt = &y_gt_arr[i];
        let loss = y_pred.binary_cross_entropy::<&Tensor>(y_gt, None, Reduction::Mean);

        loss.backward();

        (y_pred, loss)
      }).unzip();

      profiler.end("loss");

      profiler.start("backward_1");

      let mut grad_store = (0..tasks.len() * num_facts_per_task).map(|_| vs.root().zeros("z", &[]).set_requires_grad(false)).collect::<Vec<_>>();

      (0..tasks.len() as i64 * out_dim).for_each(|i| {
        let task_id = i / out_dim;
        let elem_id = i % out_dim;

        let y_pred = &y_preds[task_id as usize];
        let y_pred_grad_ith = y_pred.grad().i(elem_id);

        let var_id_to_deriv_map = &results[i as usize].2;
        for (fact_id, deriv) in var_id_to_deriv_map {
          let mult_grad = deriv.clone() * &y_pred_grad_ith;
          let index = num_facts_per_task * task_id as usize + fact_id;
          grad_store[index] += mult_grad;
        }
      });

      profiler.end("backward_1");

      profiler.start("backward_2");

      for (i, grad) in grad_store.into_iter().enumerate() {
        let task_id = i / num_facts_per_task;
        let fact_id = i % num_facts_per_task;

        let row = task_id * num_digits_per_task as usize + fact_id / num_possible_digits;
        let col = fact_id % num_possible_digits;

        raw_y_preds.i((row as i64, col as i64)).backward_with_grad(&grad, true, false);
      }

      profiler.end("backward_2");

      profiler.start("step");

      let prev_ith_task = ith_task;

      // Compute loss
      ith_task += tasks.len();
      ith_batch += 1;
      let avg_loss = Tensor::stack(&losses, 0).mean(Kind::Float);
      profiler.log(&format!("{}/{}, Batch {}, loss: {:?}", ith_task, num_tasks, ith_batch, avg_loss));

      // Back propagate
      opt.step();
      profiler.end("step");
      // profiler.log_profile_record();
      profiler.end("batch_time");

      // Test
      if prev_ith_task / test_tasks < ith_task / test_tasks {
        // Get and print test accuracy
        let test_accuracy = net.batch_accuracy_for_logits(&m.test_images, &m.test_labels, vs.device(), 1024);
        profiler.log(&format!("epoch: {:4}, batch: {:4}, test acc: {:5.2}%", epoch, ith_batch, 100. * test_accuracy));
      }
    }

    profiler.end(&epoch_name);
    profiler.log("\n");

    // Get and print test accuracy
    let test_accuracy = net.batch_accuracy_for_logits(&m.test_images, &m.test_labels, vs.device(), 1024);
    profiler.log(&format!("epoch: {:4} test acc: {:5.2}%", epoch, 100. * test_accuracy));

    // Print the profiler
    profiler.log_profile_record();
  }

  profiler.end("total_time");
  Ok(())
}

fn main() -> Result<()> {
  let opt = Options::from_args();
  if opt.k == 1 {
    run::<1>(&opt)
  } else if opt.k == 2 {
    run::<2>(&opt)
  } else if opt.k == 3 {
    run::<3>(&opt)
  } else if opt.k == 4 {
    run::<4>(&opt)
  } else if opt.k == 5 {
    run::<5>(&opt)
  } else if opt.k == 10 {
    run::<10>(&opt)
  } else if opt.k == 20 {
    run::<20>(&opt)
  } else {
    panic!("K = {} not supported", opt.k)
  }
}
