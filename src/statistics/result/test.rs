use std::num::NonZeroUsize;
use std::time::Duration;

use crate::statistics::result::construct_result;
use crate::statistics::result::{TypingResultStatistics, TypingResultStatisticsTarget};
use crate::typing_primitive_types::chunk::ChunkState;
use crate::typing_primitive_types::key_stroke::ActualKeyStroke;
use crate::LapRequest;
use crate::{gen_candidate, gen_chunk};

#[test]
fn construct_result_1() {
    let cc = vec![
        gen_chunk!(
            "きょ",
            vec![
                gen_candidate!(["kyo"], true, Some(3)),
                gen_candidate!(["ki", "lyo"], false, Some(1)),
                gen_candidate!(["ki", "xyo"], false, Some(1))
            ],
            ChunkState::Confirmed,
            gen_candidate!(["kyo"], true, None),
            [
                ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(2, 0), 'u'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(3, 0), 'u'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(4, 0), 'y'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(5, 0), 'o'.try_into().unwrap(), true)
            ]
        ),
        gen_chunk!(
            "きょ",
            vec![
                gen_candidate!(["kyo"], false, Some(1)),
                gen_candidate!(["ki", "lyo"], false, Some(2)),
                gen_candidate!(["ki", "xyo"], true, Some(5))
            ],
            ChunkState::Confirmed,
            gen_candidate!(["kyo"], true, None),
            [
                ActualKeyStroke::new(Duration::new(6, 0), 'k'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(7, 0), 'i'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(8, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(9, 0), 'x'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(10, 0), 'y'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(11, 0), 'y'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(12, 0), 'o'.try_into().unwrap(), true)
            ]
        ),
    ];

    let trs = construct_result(&cc, LapRequest::Spell(NonZeroUsize::new(1).unwrap()));

    assert_eq!(
        trs,
        TypingResultStatistics::new(
            TypingResultStatisticsTarget::new(8, 5, 4),
            TypingResultStatisticsTarget::new(6, 3, 4),
            Duration::new(12, 0),
        )
    );
}
