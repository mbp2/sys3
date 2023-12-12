#[cfg(test)]
pub fn trivial_assertion() {
   print!("A trivial assertion: ");
   assert_eq!(1, 1);
   println!("[ok]");
}
