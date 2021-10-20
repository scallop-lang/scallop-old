import os
import re

time_kws = ["wmc", "datalog_execute", "backward_1", "backward_2", "datalog_prepare", "prediction", "loss", "step"]
digits = '[0-9]+.?[0-9]*'


def collect_acc_time(file):
    all_accs = []
    all_time = []
    new_time = 0
    for line in file:
        # if "batch" in line and "acc" in line:
        if "=-=" in line and "Batch" in line:
            numbers = re.findall(digits, line)
            epoch_id = numbers[0]
            batch_id = numbers[1]
            loss = numbers[2]
            acc = numbers[3]
            all_accs.append((epoch_id, batch_id, acc))
        if "Profiler record:" in line:
            all_time.append(new_time)
            new_time = 0
        for kw in time_kws:
            if kw in line:
                new_line = line.split(':')
                numbers = re.findall(digits, new_line[1])
                new_time += float(numbers[0])
    all_time.append(new_time)
    return all_accs, all_time


def print_acc(accs):
    for ct, acc in enumerate(accs):
        # print(f"{ct * 1000}\t{acc[-1]}")
         print(f"{acc[-1]}")


if __name__ == "__main__":
    log_dir = os.path.abspath(os.path.join(os.path.abspath(__file__), "../../log"))
    digit = 2
    task = f"sum_{digit}_decouple_ce"
    train_k = 1
    test_k = 1
    k = 15
    lr = 0.001
    batch_size = 64
    epoch = None
    seed = 1234

    # if epoch is not None:
    #     log_name = f"{task}_k_{k}_lr_{lr}_bs_{batch_size}_e_{epoch}_rs_{seed}.log"
    # else:
    #     log_name = f"{task}_k_{k}_lr_{lr}_bs_{batch_size}_rs_{seed}.log"

    log_name = f"{task}_trk_{train_k}_tek_{test_k}_lr_{lr}_bs_{batch_size}_rs_{seed}.log"
    log_path = os.path.join(log_dir, log_name)

    with open (log_path, 'r') as log_file:
        accs, times = collect_acc_time(log_file)
        avg_time = times[-1] / (len(accs) * 1000)
        print_acc(accs)
        # print(avg_time)
