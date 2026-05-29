// Copyright (c) 2019-2026 Provable Inc.
// This file is part of the snarkOS library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:

// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Generates a string containing all continuous sequenences within the list of numbers.
///
/// The function expects the input to already be sorted.
pub fn rangify_heights(heights: impl IntoIterator<Item = u32>) -> String {
    let mut iter = heights.into_iter().peekable();
    let mut ret = String::from("[");
    let mut first = true;

    while let Some(next_height) = iter.next() {
        let start = next_height;
        let mut curr_height = start;

        while let Some(next_height) = iter.peek().copied() {
            if next_height == curr_height + 1 {
                curr_height += 1;
                let _ = iter.next();
            } else {
                break;
            }
        }

        if first {
            first = false;
        } else {
            ret.push_str(", ");
        }

        if curr_height == start {
            ret.push_str(&format!("{curr_height}"));
        } else {
            ret.push_str(&format!("{start}-{curr_height}"));
        }
    }

    ret.push(']');
    ret
}

#[cfg(test)]
mod tests {
    use super::rangify_heights;

    #[test]
    fn test_rangify_heights_empty() {
        let heights: &[u32] = &[];

        let rangified = rangify_heights(heights.iter().copied());
        assert_eq!(rangified, "[]");
    }

    #[test]
    fn test_rangify_heights_single_value() {
        let heights = &[52425];

        let rangified = rangify_heights(heights.iter().copied());
        assert_eq!(rangified, "[52425]");
    }

    #[test]
    fn test_rangify_large_continuous() {
        let start = 16353;
        let end = start + 52414;

        let heights = start..=end;

        let rangified = rangify_heights(heights);
        assert_eq!(rangified, format!("[{start}-{end}]"));
    }

    #[test]
    fn test_rangify_many_small() {
        let mut heights = vec![];
        let mut expected = vec![];

        for idx in 0..100 {
            let start = idx * 4;
            let mid = idx * 4 + 1;
            let end = idx * 4 + 2;
            expected.push(format!("{start}-{end}"));
            heights.push(start);
            heights.push(mid);
            heights.push(end);
        }

        let rangified = rangify_heights(heights);
        let expected = format!("[{}]", expected.join(", "));

        assert_eq!(rangified, expected);
    }

    #[test]
    fn test_rangify_heights_multiple_ranges() {
        let heights = &[0, 1, 3, 4, 5, 6, 7, 10, 11, 12, 20, 22, 23, 24, 25, 27, 28];

        let rangified = rangify_heights(*heights);
        assert_eq!(rangified, "[0-1, 3-7, 10-12, 20, 22-25, 27-28]");
    }
}
