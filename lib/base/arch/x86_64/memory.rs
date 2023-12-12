static mut ROOT_PAGE_TABLE: usize = 0;

#[inline(always)]
pub fn get_kernel_root_page_table() -> usize {
   return unsafe{ ROOT_PAGE_TABLE };
}
