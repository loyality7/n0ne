pub(crate) fn get_builtin_mapping(name: &str) -> Option<(&'static str, bool)> {
    match name {
        "show" => Some(("show", true)),
        "c_alloc" => Some(("n0_c_alloc", true)),
        "c_store_int" => Some(("n0_c_store_int", true)),
        "c_store_string" => Some(("n0_c_store_string", true)),
        "c_load_int" => Some(("n0_c_load_int", true)),
        "c_load_string" => Some(("n0_c_load_string", true)),
        "c_interpolate" => Some(("n0_c_interpolate", true)),
        "c_argc" => Some(("n0_c_argc", true)),
        "c_argv" => Some(("n0_c_argv", true)),
        "some" => Some(("n0_make_some", true)),
        "none" => Some(("n0_make_none", true)),
        "ok" => Some(("n0_make_ok", true)),
        "err" => Some(("n0_make_err", true)),
        _ => None,
    }
}
