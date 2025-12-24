fn main() {
    // Declare the swc_ast_unknown cfg flag to avoid unexpected_cfgs warnings
    println!("cargo::rustc-check-cfg=cfg(swc_ast_unknown)");
}
