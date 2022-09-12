pub type Result = core::result::Result<(), &dyn Error>;

pub trait Error {
   fn msg() -> &str;
}
