#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
   crate::println!("Running {} tests", tests.len());
   for test in tests {
      test();
   }
}
