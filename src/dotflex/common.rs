static mut OUTPUT_VERBOSE : bool = false;

pub fn set_output_verbosity(verbose: bool) {
    unsafe {
        OUTPUT_VERBOSE = verbose;
    }
}

pub fn output_verbose() -> bool {
    unsafe {
        OUTPUT_VERBOSE
    }
}
