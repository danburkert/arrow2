use crate::error::{ArrowError, Result};
use crate::types::Offset;

pub fn check_offsets_minimal<O: Offset>(offsets: &[O], values_len: usize) -> usize {
    assert!(
        !offsets.is_empty(),
        "The length of the offset buffer must be larger than 1"
    );
    let len = offsets.len() - 1;

    let last_offset = offsets[len];
    let last_offset = last_offset.to_usize();

    assert_eq!(
        values_len, last_offset,
        "The length of the values must be equal to the last offset value"
    );
    len
}

/// # Panics iff:
/// * the `offsets` is not monotonically increasing, or
/// * any slice of `values` between two consecutive pairs from `offsets` is invalid `utf8`, or
/// * any offset is larger or equal to `values_len`.
pub fn check_offsets_and_utf8<O: Offset>(offsets: &[O], values: &[u8]) {
    try_check_offsets_and_utf8(offsets, values).unwrap()
}

/// # Panics iff:
/// * the `offsets` is not monotonically increasing, or
/// * any slice of `values` between two consecutive pairs from `offsets` is invalid `utf8`, or
/// * any offset is larger or equal to `values_len`.
pub fn try_check_offsets_and_utf8<O: Offset>(offsets: &[O], values: &[u8]) -> Result<()> {
    const SIMD_CHUNK_SIZE: usize = 64;
    if values.is_ascii() {
        try_check_offsets(offsets, values.len())
    } else {
        for window in offsets.windows(2) {
            let start = window[0].to_usize();
            let end = window[1].to_usize();

            // check monotonicity
            if start > end {
                return Err(ArrowError::oos("offsets must be monotonically increasing"));
            }

            // check bounds
            if end > values.len() {
                return Err(ArrowError::oos("offsets must not exceed values length"));
            };

            let slice = &values[start..end];

            // fast ASCII check per item
            if slice.len() < SIMD_CHUNK_SIZE && slice.is_ascii() {
                continue;
            }

            // check utf8
            simdutf8::basic::from_utf8(slice)?;
        }

        Ok(())
    }
}

/// # Panics iff:
/// * the `offsets` is not monotonically increasing, or
/// * any offset is larger or equal to `values_len`.
pub fn check_offsets<O: Offset>(offsets: &[O], values_len: usize) {
    try_check_offsets(offsets, values_len).unwrap()
}

/// Checks that `offsets` is monotonically increasing, and all offsets are less than or equal to
/// `values_len`.
pub fn try_check_offsets<O: Offset>(offsets: &[O], values_len: usize) -> Result<()> {
    if offsets.windows(2).any(|window| window[0] > window[1]) {
        Err(ArrowError::oos("offsets must be monotonically increasing"))
    } else if offsets
        .last()
        .map_or(false, |last| last.to_usize() > values_len)
    {
        Err(ArrowError::oos("offsets must not exceed values length"))
    } else {
        Ok(())
    }
}
