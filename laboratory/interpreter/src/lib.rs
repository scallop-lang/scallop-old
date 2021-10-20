mod component;
mod element;
mod relation;
mod utils;

pub use component::*;
pub use element::*;
pub use relation::*;

// #[derive(Debug, Clone)]
// pub struct ElementHandleMut<'a, Tag: Semiring> {
//   tag: &'a Tag,
//   comp_type: &'a ComponentType,
//   comp_pointer: *mut u8,
// }

// use scallop_runtime::Semiring;

// /// Component accessor into tuples.
// ///
// /// # Examples
// ///
// /// Given a tuple (A, B), indices [0] gives A and [1] gives B
// ///
// /// Given a "tuple" A (essentially a component itself), indices [] gives A
// ///
// /// Given a nested tuple like (A, (B, C), D, (E, F)), [0] gives A,
// /// [1, 0] gives B, [1, 1] gives C, and so on.
// ///
// /// # Note
// ///
// /// We assume there are maximum 4 levels of nesting; and each tuple does not
// /// contain more than 256 components.
// #[derive(Debug, Clone)]
// pub struct ComponentAccessor {
//   size: u8,
//   indices: [u8; 3],
// }

// impl ComponentAccessor {
//   pub fn root() -> Self {
//     Self {
//       size: 0,
//       indices: [0; 3],
//     }
//   }

//   pub fn accessor(&self) -> &[u8] {
//     &self.indices[0..self.size as usize]
//   }

//   pub fn new(indices: &[u8]) -> Self {
//     let mut new_indices = [0u8; 3];
//     for i in 0..indices.len() {
//       new_indices[i] = indices[i];
//     }
//     Self {
//       size: indices.len() as u8,
//       indices: new_indices,
//     }
//   }

//   pub fn indent(&self) -> Self {
//     Self::new(&self.indices[1..])
//   }
// }

// #[derive(Debug, Clone)]
// pub enum Component {
//   Integer(i64),
//   Boolean(bool),
//   Symbol(usize),
//   String(String),
//   Tuple(Vec<Component>),
// }

// #[derive(Clone, Debug)]
// pub enum ComponentType {
//   Integer,
//   Boolean,
//   Symbol,
//   String,
//   Tuple(Vec<ComponentType>),
// }

// impl ComponentType {
//   pub fn size(&self) -> usize {
//     match self {
//       Self::Boolean => std::mem::size_of::<bool>(),
//       Self::Integer => std::mem::size_of::<i64>(),
//       Self::Symbol => std::mem::size_of::<usize>(),
//       Self::String => std::mem::size_of::<String>(),
//       Self::Tuple(elems) => elems.iter().map(Self::size).sum(),
//     }
//   }

//   pub fn type_of_component_indices(&self, acc: &[u8]) -> ComponentType {
//     if acc.len() == 0 {
//       self.clone()
//     } else {
//       if let Self::Tuple(types) = &self {
//         let idx = acc[0] as usize;
//         types[idx].type_of_component_indices(&acc[1..])
//       } else {
//         panic!("Invalid accessor {:?}", acc)
//       }
//     }
//   }

//   pub fn type_of_component(&self, acc: &ComponentAccessor) -> ComponentType {
//     self.type_of_component_indices(&acc.indices[0..acc.size as usize])
//   }

//   fn offset_of_indices(&self, acc: &[u8]) -> usize {
//     if acc.len() == 0 {
//       0
//     } else {
//       if let Self::Tuple(types) = &self {
//         let idx = acc[0] as usize;
//         let prior : usize = types[0..idx].iter().map(Self::size).sum();
//         prior + types[idx].offset_of_indices(&acc[1..])
//       } else {
//         panic!("Invalid accessor {:?}", acc);
//       }
//     }
//   }

//   pub fn offset(&self, acc: &ComponentAccessor) -> usize {
//     self.offset_of_indices(&acc.indices[0..acc.size as usize])
//   }
// }

// pub trait ElementHandle<Tag: Semiring> {
//   fn component(&self, acc: &ComponentAccessor) -> Component;

//   fn tag(&self) -> &Tag;
// }

// pub trait ElementHandleMut<Tag: Semiring> : ElementHandle<Tag> {
//   fn set_component(&mut self, acc: &ComponentAccessor, data: Component);

//   fn set_tag(&mut self, tag: Tag);
// }

// pub trait Relation<'a, Tag: Semiring> { // : IntoIterator<Item = (Component, Tag)> {
//   type Handle : ElementHandle<Tag>;

//   type HandleMut : ElementHandleMut<Tag>;

//   fn empty() -> Self;

//   fn push(&'a mut self, element: Component, tag: Tag);

//   fn from_vec_unchecked(elements: Vec<(Component, Tag)>) -> Self;

//   fn from_vec(elements: Vec<(Component, Tag)>, ctx: &Tag::Context) -> Self;

//   fn len(&'a self) -> usize;

//   fn is_empty(&'a self) -> bool;

//   fn get(&'a self, i: usize) -> Self::Handle;

//   fn get_mut(&'a mut self, i: usize) -> Self::HandleMut;

//   fn merge(self, other: Self, ctx: &Tag::Context) -> Self;
// }

// pub struct DynamicElementHandle<'a, Tag: Semiring> {
//   pub tag: &'a Tag,
//   ty: &'a ComponentType,
//   pointer: *const u8,
// }

// impl<'a, Tag: Semiring> ElementHandle<Tag> for DynamicElementHandle<'a, Tag> {
//   fn component(&self, acc: &ComponentAccessor) -> Component {
//     let ty = self.ty.type_of_component(acc);
//     let offset = self.ty.offset(acc);
//     let loc = self.pointer as usize + offset;
//     match ty {
//       ComponentType::Boolean => {
//         unsafe { Component::Boolean(*(loc as *const bool)) }
//       }
//       ComponentType::Integer => {
//         unsafe { Component::Integer(*(loc as *const i64)) }
//       }
//       ComponentType::String => {
//         unsafe { Component::String((*(loc as *const String)).clone()) }
//       }
//       ComponentType::Symbol => {
//         unsafe { Component::Symbol(*(loc as *const usize)) }
//       }
//       ComponentType::Tuple(elem_tys) => {
//         // match &acc.indices[0..acc.size as usize] {
//         //   [] => {
//         //     let (_, elems) = elem_tys.iter().fold((0, vec![]), |(offset, elems), curr_ty| {
//         //       (offset, vec![])
//         //     });
//         //     Component::Tuple(elems)
//         //   }
//         //   indices => {
//         //     let elem_ty = &elem_tys[indices[0] as usize];
//         //     let handle = Self {
//         //       tag: self.tag,
//         //       ty: elem_ty,
//         //       pointer: (self.pointer as usize + elem_ty.size()) as *const u8,
//         //     };
//         //     handle.component(&acc.indent())
//         //   }
//         // }
//         panic!("Hmmm")
//       }
//     }
//   }

//   fn tag(&self) -> &Tag {
//     self.tag
//   }
// }

// pub struct DynamicRelation<Tag: Semiring> {
//   pub ty: ComponentType,
//   pub elem_view: Vec<u8>,
//   pub elem_size: usize,
//   pub tags: Vec<Tag>,
// }

// // impl<'a, Tag: Semiring> Relation<'a, Tag> for DynamicRelation<Tag> {
// //   type Handle = DynamicElementHandle<'a, Tag>;

// //   fn get(&'a self, i: usize) -> Self::Handle {
// //     Self::Handle {
// //       tag: &self.tags[i],
// //       ty: &self.ty,
// //       view_pointer: (&self.elem_view[self.elem_size * i]) as *const u8,
// //     }
// //   }

// //   fn push(&'a mut self, elem: Component, tag: Tag) {
// //     self.elem_view.extend(vec![0; self.elem_size]);
// //     Relation::get_mut(self, self.len()).set_component(ComponentAccessor::root(), elem);
// //     self.tags.push(tag);
// //   }
// // }

// #[cfg(test)]
// mod dynamic_type_test {
//   use super::*;

//   #[test]
//   fn test_dynamic_type_1() {
//     let ty = ComponentType::Integer;
//     assert_eq!(ty.size(), 8);
//     assert_eq!(ty.offset(&ComponentAccessor::new(&[])), 0);
//   }

//   #[test]
//   fn test_dynamic_type_2() {
//     let ty = ComponentType::Tuple(vec![ComponentType::Boolean, ComponentType::Boolean, ComponentType::Integer]);
//     assert_eq!(ty.size(), 10);
//     assert_eq!(ty.offset(&ComponentAccessor::new(&[])), 0);
//     assert_eq!(ty.offset(&ComponentAccessor::new(&[0])), 0);
//     assert_eq!(ty.offset(&ComponentAccessor::new(&[1])), 1);
//     assert_eq!(ty.offset(&ComponentAccessor::new(&[2])), 2);
//   }

//   #[test]
//   fn match_list() {
//     let arr : Vec<usize> = vec![0, 0, 1];
//     match arr[..] {
//       [0, 0] => println!("0, 0"),
//       [0, 1] => println!("0, 1"),
//       [0, 0, 1] => println!("0, 0, 1"),
//       _ => {}
//     }
//   }
// }
