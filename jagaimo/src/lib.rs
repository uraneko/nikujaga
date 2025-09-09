use jagaimo_macro::jagaimo;

jagaimo! {
    #[
        nu_cmp,
        fish_cmp,
        root_name = "jagaimo_n",
        // ignore_naming_conventions,
        // no_auto_alias,
        derives(),
    ]

    o(add) = A
    s(remote) = rmt
    s(collections) = colls

    c { s(hosts) o(view) [ <i32> filter<String> colored query<String> ] }
    c { s(history) o(view) [ flatten encoding<String> max<u8> max<f64> ] }
    c { s(history) o(annotate) [ max<u8> verbose tags<Vec<String>> ] }
    c { [ <(String, f64)> size<Dimensions> show_all ] }
    c { s(collections) o(obfuscate) [ <String> allocate  rand<f64> hash<String> fuzzing algorithm<String> ] }
    c { o(view) [ list_all theme<String> ] }
    c { o(distribute) }
    c { s(collections) o(list) [ <Vec<f64>> long pipe_into_list_of<String> output_file<String> ] }


    // t { s(colls) o(list) |_ base: String|
    //         { if base == "_" { auto as bool } else { base "BASE{base}" as Base } }
    // }
    // t { s(history) |use_max| { use 453 as u32} }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Dimensions {
    x: u8,
    y: u8,
}
