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
                    gen_candidate!(["jo"], true, None),
                    gen_candidate!(["zyo"], true, None),
                    gen_candidate!(["jyo"], true, None),
                    gen_candidate!(["zi", "lyo"], true, None),
                    gen_candidate!(["zi", "xyo"], true, None),
                    gen_candidate!(["ji", "lyo"], true, None),
                    gen_candidate!(["ji", "xyo"], true, None),
                ],
                gen_candidate!(["jo"], true, None)
            ),
            gen_chunk!(
                "ん",
                vec![
                    gen_candidate!(["nn"], true, None),
                    gen_candidate!(["xn"], true, None),
                ],
                gen_candidate!(["nn"], true, None)
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
                    gen_candidate!(["u"], true, None),
                    gen_candidate!(["wu"], true, None),
                    gen_candidate!(["whu"], true, None)
                ],
                gen_candidate!(["u"], true, None)
            ),
            gen_chunk!(
                "っ",
                vec![
                    gen_candidate!(["w"], true, None, 'w'),
                    gen_candidate!(["ltu"], true, None),
                    gen_candidate!(["xtu"], true, None),
                    gen_candidate!(["ltsu"], true, None)
                ],
                gen_candidate!(["w"], true, None, 'w')
            ),
            gen_chunk!(
                "う",
                vec![
                    gen_candidate!(["u"], true, None),
                    gen_candidate!(["wu"], true, None),
                    gen_candidate!(["whu"], true, None)
                ],
                gen_candidate!(["wu"], true, None)
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
                vec![
                    gen_candidate!(["ka"], true, None),
                    gen_candidate!(["ca"], true, None)
                ],
                gen_candidate!(["ka"], true, None)
            ),
            gen_chunk!(
                "ん",
                vec![
                    gen_candidate!(["n"], true, None, ['z', 'j']),
                    gen_candidate!(["nn"], true, None),
                    gen_candidate!(["xn"], true, None)
                ],
                gen_candidate!(["n"], true, None, ['z', 'j'])
            ),
            gen_chunk!(
                "じ",
                vec![
                    gen_candidate!(["zi"], true, None),
                    gen_candidate!(["ji"], true, None)
                ],
                gen_candidate!(["zi"], true, None)
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
            gen_chunk!(
                "B",
                vec![gen_candidate!(["B"], true, None)],
                gen_candidate!(["B"], true, None)
            ),
            gen_chunk!(
                "i",
                vec![gen_candidate!(["i"], true, None)],
                gen_candidate!(["i"], true, None)
            ),
            gen_chunk!(
                "g",
                vec![gen_candidate!(["g"], true, None)],
                gen_candidate!(["g"], true, None)
            ),
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
                    gen_candidate!(["l"], true, None, 'l', ['l']),
                    gen_candidate!(["x"], true, None, 'x', ['x']),
                    gen_candidate!(["ltu"], true, None),
                    gen_candidate!(["xtu"], true, None),
                    gen_candidate!(["ltsu"], true, None),
                ],
                gen_candidate!(["l"], true, None, 'l', ['l'])
            ),
            gen_chunk!(
                "っ",
                vec![
                    gen_candidate!(["ltu"], true, None),
                    gen_candidate!(["xtu"], true, None),
                    gen_candidate!(["ltsu"], true, None),
                ],
                gen_candidate!(["ltu"], true, None)
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
                    gen_candidate!(["k"], true, None, 'k'),
                    gen_candidate!(["c"], true, None, 'c'),
                    gen_candidate!(["ltu"], true, None),
                    gen_candidate!(["xtu"], true, None),
                    gen_candidate!(["ltsu"], true, None),
                ],
                gen_candidate!(["k"], true, None, 'k')
            ),
            gen_chunk!(
                "か",
                vec![
                    gen_candidate!(["ka"], true, None),
                    gen_candidate!(["ca"], true, None),
                ],
                gen_candidate!(["ka"], true, None)
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
                vec![
                    gen_candidate!(["i"], true, None),
                    gen_candidate!(["yi"], true, None),
                ],
                gen_candidate!(["i"], true, None)
            ),
            gen_chunk!(
                "ん",
                vec![
                    gen_candidate!(["n"], true, None, ['s', 'c']),
                    gen_candidate!(["nn"], true, None),
                    gen_candidate!(["xn"], true, None)
                ],
                gen_candidate!(["n"], true, None, ['s', 'c'])
            ),
            gen_chunk!(
                "しょ",
                vec![
                    gen_candidate!(["syo"], true, None),
                    gen_candidate!(["sho"], true, None),
                    gen_candidate!(["si", "lyo"], true, None),
                    gen_candidate!(["si", "xyo"], true, None),
                    gen_candidate!(["ci", "lyo"], true, None),
                    gen_candidate!(["ci", "xyo"], true, None),
                    gen_candidate!(["shi", "lyo"], true, None),
                    gen_candidate!(["shi", "xyo"], true, None),
                ],
                gen_candidate!(["syo"], true, None)
            ),
            gen_chunk!(
                "う",
                vec![
                    gen_candidate!(["u"], true, None),
                    gen_candidate!(["wu"], true, None),
                    gen_candidate!(["whu"], true, None)
                ],
                gen_candidate!(["u"], true, None)
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
                    gen_candidate!(["n"], true, None, 'w', ['w']),
                    gen_candidate!(["nn"], true, None),
                    gen_candidate!(["xn"], true, None),
                ],
                gen_candidate!(["n"], true, None, 'w', ['w'])
            ),
            gen_chunk!(
                "う",
                vec![
                    gen_candidate!(["u"], true, None),
                    gen_candidate!(["wu"], true, None),
                    gen_candidate!(["whu"], true, None)
                ],
                gen_candidate!(["wu"], true, None)
            ),
        ]
    );
}

#[test]
fn strict_key_stroke_count_1() {
    let mut chunk = gen_chunk!(
        "じょ",
        vec![
            gen_candidate!(["jo"], true, None),
            gen_candidate!(["zyo"], true, None),
            gen_candidate!(["jyo"], true, None),
            gen_candidate!(["zi", "lyo"], true, None),
            gen_candidate!(["zi", "xyo"], true, None),
            gen_candidate!(["ji", "lyo"], true, None),
            gen_candidate!(["ji", "xyo"], true, None),
        ],
        gen_candidate!(["jo"], true, None)
    );

    chunk.strict_key_stroke_count(NonZeroUsize::new(1).unwrap());

    assert_eq!(
        chunk,
        gen_chunk!(
            "じょ",
            vec![
                gen_candidate!(["j"], true, None),
                gen_candidate!(["z"], true, None),
            ],
            gen_candidate!(["j"], true, None)
        )
    )
}

#[test]
fn strict_key_stroke_count_2() {
    let mut chunk = gen_chunk!(
        "ん",
        vec![
            gen_candidate!(["n"], true, None, ['j', 'z']),
            gen_candidate!(["nn"], true, None),
            gen_candidate!(["xn"], true, None),
        ],
        gen_candidate!(["n"], true, None, ['j', 'z'])
    );

    chunk.strict_key_stroke_count(NonZeroUsize::new(1).unwrap());

    assert_eq!(
        chunk,
        gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["n"], true, None),
                gen_candidate!(["x"], true, None)
            ],
            gen_candidate!(["n"], true, None)
        )
    )
}

#[test]
fn is_element_end_at_key_stroke_index_1() {
    let c = gen_candidate!(["ki", "xyo"], true, None);

    assert!(!c.is_element_end_at_key_stroke_index(0));
    assert!(c.is_element_end_at_key_stroke_index(1));
    assert!(!c.is_element_end_at_key_stroke_index(2));
    assert!(!c.is_element_end_at_key_stroke_index(3));
    assert!(c.is_element_end_at_key_stroke_index(4));
}
