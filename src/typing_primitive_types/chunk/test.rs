use super::*;

use crate::{gen_candidate, gen_chunk, gen_unprocessed_chunk};

#[test]
fn append_key_stroke_to_chunks_1() {
    let mut chunks = vec![gen_unprocessed_chunk!("じょ"), gen_unprocessed_chunk!("ん")];

    append_key_stroke_to_chunks(&mut chunks);

    assert_eq!(
        chunks,
        vec![
            gen_chunk!(
                "じょ",
                vec![
                    gen_candidate!(["jo"]),
                    gen_candidate!(["zyo"]),
                    gen_candidate!(["jyo"]),
                    gen_candidate!(["zi", "lyo"]),
                    gen_candidate!(["zi", "xyo"]),
                    gen_candidate!(["ji", "lyo"]),
                    gen_candidate!(["ji", "xyo"]),
                ],
                gen_candidate!(["jo"])
            ),
            gen_chunk!(
                "ん",
                vec![gen_candidate!(["nn"]), gen_candidate!(["xn"])],
                gen_candidate!(["nn"])
            )
        ]
    );
}

#[test]
fn append_key_stroke_to_chunks_2() {
    let mut chunks = vec![
        gen_unprocessed_chunk!("う"),
        gen_unprocessed_chunk!("っ"),
        gen_unprocessed_chunk!("う"),
    ];

    append_key_stroke_to_chunks(&mut chunks);

    assert_eq!(
        chunks,
        vec![
            gen_chunk!(
                "う",
                vec![
                    gen_candidate!(["u"]),
                    gen_candidate!(["wu"]),
                    gen_candidate!(["whu"])
                ],
                gen_candidate!(["u"])
            ),
            gen_chunk!(
                "っ",
                vec![
                    gen_candidate!(["w"], 'w'),
                    gen_candidate!(["ltu"]),
                    gen_candidate!(["xtu"]),
                    gen_candidate!(["ltsu"])
                ],
                gen_candidate!(["w"], 'w')
            ),
            gen_chunk!(
                "う",
                vec![
                    gen_candidate!(["u"]),
                    gen_candidate!(["wu"]),
                    gen_candidate!(["whu"])
                ],
                gen_candidate!(["wu"])
            ),
        ]
    );
}

#[test]
fn append_key_stroke_to_chunks_3() {
    let mut chunks = vec![
        gen_unprocessed_chunk!("か"),
        gen_unprocessed_chunk!("ん"),
        gen_unprocessed_chunk!("じ"),
    ];

    append_key_stroke_to_chunks(&mut chunks);

    assert_eq!(
        chunks,
        vec![
            gen_chunk!(
                "か",
                vec![gen_candidate!(["ka"]), gen_candidate!(["ca"])],
                gen_candidate!(["ka"])
            ),
            gen_chunk!(
                "ん",
                vec![
                    gen_candidate!(["n"], ['z', 'j']),
                    gen_candidate!(["nn"]),
                    gen_candidate!(["xn"])
                ],
                gen_candidate!(["n"], ['z', 'j'])
            ),
            gen_chunk!(
                "じ",
                vec![gen_candidate!(["zi"]), gen_candidate!(["ji"])],
                gen_candidate!(["zi"])
            ),
        ]
    );
}

#[test]
fn append_key_stroke_to_chunks_4() {
    let mut chunks = vec![
        gen_unprocessed_chunk!("B"),
        gen_unprocessed_chunk!("i"),
        gen_unprocessed_chunk!("g"),
    ];

    append_key_stroke_to_chunks(&mut chunks);

    assert_eq!(
        chunks,
        vec![
            gen_chunk!("B", vec![gen_candidate!(["B"])], gen_candidate!(["B"])),
            gen_chunk!("i", vec![gen_candidate!(["i"])], gen_candidate!(["i"])),
            gen_chunk!("g", vec![gen_candidate!(["g"])], gen_candidate!(["g"])),
        ]
    );
}

#[test]
fn append_key_stroke_to_chunks_5() {
    let mut chunks = vec![gen_unprocessed_chunk!("っ"), gen_unprocessed_chunk!("っ")];

    append_key_stroke_to_chunks(&mut chunks);

    assert_eq!(
        chunks,
        vec![
            gen_chunk!(
                "っ",
                vec![
                    gen_candidate!(["l"], 'l', ['l']),
                    gen_candidate!(["x"], 'x', ['x']),
                    gen_candidate!(["ltu"]),
                    gen_candidate!(["xtu"]),
                    gen_candidate!(["ltsu"]),
                ],
                gen_candidate!(["l"], 'l', ['l'])
            ),
            gen_chunk!(
                "っ",
                vec![
                    gen_candidate!(["ltu"]),
                    gen_candidate!(["xtu"]),
                    gen_candidate!(["ltsu"]),
                ],
                gen_candidate!(["ltu"])
            ),
        ]
    );
}

#[test]
fn append_key_stroke_to_chunks_6() {
    let mut chunks = vec![gen_unprocessed_chunk!("っ"), gen_unprocessed_chunk!("か")];

    append_key_stroke_to_chunks(&mut chunks);

    assert_eq!(
        chunks,
        vec![
            gen_chunk!(
                "っ",
                vec![
                    gen_candidate!(["k"], 'k'),
                    gen_candidate!(["c"], 'c'),
                    gen_candidate!(["ltu"]),
                    gen_candidate!(["xtu"]),
                    gen_candidate!(["ltsu"]),
                ],
                gen_candidate!(["k"], 'k')
            ),
            gen_chunk!(
                "か",
                vec![gen_candidate!(["ka"]), gen_candidate!(["ca"]),],
                gen_candidate!(["ka"])
            ),
        ]
    );
}

#[test]
fn append_key_stroke_to_chunks_7() {
    let mut chunks = vec![
        gen_unprocessed_chunk!("い"),
        gen_unprocessed_chunk!("ん"),
        gen_unprocessed_chunk!("しょ"),
        gen_unprocessed_chunk!("う"),
    ];

    append_key_stroke_to_chunks(&mut chunks);

    assert_eq!(
        chunks,
        vec![
            gen_chunk!(
                "い",
                vec![gen_candidate!(["i"]), gen_candidate!(["yi"]),],
                gen_candidate!(["i"])
            ),
            gen_chunk!(
                "ん",
                vec![
                    gen_candidate!(["n"], ['s', 'c']),
                    gen_candidate!(["nn"]),
                    gen_candidate!(["xn"])
                ],
                gen_candidate!(["n"], ['s', 'c'])
            ),
            gen_chunk!(
                "しょ",
                vec![
                    gen_candidate!(["syo"]),
                    gen_candidate!(["sho"]),
                    gen_candidate!(["si", "lyo"]),
                    gen_candidate!(["si", "xyo"]),
                    gen_candidate!(["ci", "lyo"]),
                    gen_candidate!(["ci", "xyo"]),
                    gen_candidate!(["shi", "lyo"]),
                    gen_candidate!(["shi", "xyo"]),
                ],
                gen_candidate!(["syo"])
            ),
            gen_chunk!(
                "う",
                vec![
                    gen_candidate!(["u"]),
                    gen_candidate!(["wu"]),
                    gen_candidate!(["whu"])
                ],
                gen_candidate!(["u"])
            ),
        ]
    );
}

#[test]
fn append_key_stroke_to_chunks_8() {
    let mut chunks = vec![gen_unprocessed_chunk!("ん"), gen_unprocessed_chunk!("う")];

    append_key_stroke_to_chunks(&mut chunks);

    assert_eq!(
        chunks,
        vec![
            gen_chunk!(
                "ん",
                vec![
                    gen_candidate!(["n"], 'w', ['w']),
                    gen_candidate!(["nn"]),
                    gen_candidate!(["xn"]),
                ],
                gen_candidate!(["n"], 'w', ['w'])
            ),
            gen_chunk!(
                "う",
                vec![
                    gen_candidate!(["u"]),
                    gen_candidate!(["wu"]),
                    gen_candidate!(["whu"])
                ],
                gen_candidate!(["wu"])
            ),
        ]
    );
}

#[test]
fn strict_key_stroke_count_1() {
    let mut chunk = gen_chunk!(
        "じょ",
        vec![
            gen_candidate!(["jo"]),
            gen_candidate!(["zyo"]),
            gen_candidate!(["jyo"]),
            gen_candidate!(["zi", "lyo"]),
            gen_candidate!(["zi", "xyo"]),
            gen_candidate!(["ji", "lyo"]),
            gen_candidate!(["ji", "xyo"]),
        ],
        gen_candidate!(["jo"])
    );

    chunk.strict_key_stroke_count(NonZeroUsize::new(1).unwrap());

    assert_eq!(
        chunk,
        gen_chunk!(
            "じょ",
            vec![gen_candidate!(["j"]), gen_candidate!(["z"]),],
            gen_candidate!(["j"])
        )
    )
}

#[test]
fn strict_key_stroke_count_2() {
    let mut chunk = gen_chunk!(
        "ん",
        vec![
            gen_candidate!(["n"], ['j', 'z']),
            gen_candidate!(["nn"]),
            gen_candidate!(["xn"]),
        ],
        gen_candidate!(["n"], ['j', 'z'])
    );

    chunk.strict_key_stroke_count(NonZeroUsize::new(1).unwrap());

    assert_eq!(
        chunk,
        gen_chunk!(
            "ん",
            vec![gen_candidate!(["n"]), gen_candidate!(["x"])],
            gen_candidate!(["n"])
        )
    )
}

#[test]
fn is_element_end_at_key_stroke_index_1() {
    let c = gen_candidate!(["ki", "xyo"]);

    assert!(!c.is_element_end_at_key_stroke_index(0));
    assert!(c.is_element_end_at_key_stroke_index(1));
    assert!(!c.is_element_end_at_key_stroke_index(2));
    assert!(!c.is_element_end_at_key_stroke_index(3));
    assert!(c.is_element_end_at_key_stroke_index(4));
}
