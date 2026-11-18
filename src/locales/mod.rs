pub(crate) mod en;
pub(crate) mod eo;

use crate::token::Token;
use std::collections::HashSet;

/// Turns the run of numbers right before a weekday into ordinal positions.
///
/// Call this before pushing the weekday token. `ordinals` holds the indices of tokens that were
/// written as ordinals (2nd), `after_every` the index of an Interval that directly follows
/// "every". MonthDays and ordinal Intervals in the run become positions. A single ordinal of 2 or
/// more right after "every" stays an Interval, since "every second monday" means every other
/// Monday.
pub(crate) fn mark_positions(
    tokens: &mut [Token],
    ordinals: &HashSet<usize>,
    after_every: Option<usize>,
) {
    let mut start = tokens.len();
    while start > 0 {
        match tokens[start - 1] {
            Token::MonthDay(_) | Token::OrdinalPosition(_) => {}
            Token::Interval(_) if ordinals.contains(&(start - 1)) => {}
            _ => break,
        }
        start -= 1;
    }

    let run = &mut tokens[start..];
    if let [Token::Interval(n)] = run
        && *n >= 2
        && after_every == Some(start)
    {
        return;
    }

    for t in run {
        match *t {
            Token::MonthDay(n) => *t = Token::OrdinalPosition(n as i32),
            Token::Interval(n) => *t = Token::OrdinalPosition(n as i32),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mark_positions;
    use crate::token::Token::*;
    use std::collections::HashSet;

    #[test]
    fn monthdays_become_positions() {
        let mut tokens = vec![MonthDay(1), MonthDay(3)];
        mark_positions(&mut tokens, &HashSet::new(), None);
        assert_eq!(tokens, vec![OrdinalPosition(1), OrdinalPosition(3)]);
    }

    #[test]
    fn ordinal_intervals_become_positions() {
        let mut tokens = vec![Interval(1), Interval(3)];
        mark_positions(&mut tokens, &HashSet::from([0, 1]), Some(0));
        assert_eq!(tokens, vec![OrdinalPosition(1), OrdinalPosition(3)]);
    }

    #[test]
    fn cardinal_interval_is_kept() {
        let mut tokens = vec![Interval(2)];
        mark_positions(&mut tokens, &HashSet::new(), Some(0));
        assert_eq!(tokens, vec![Interval(2)]);
    }

    #[test]
    fn single_ordinal_after_every_is_kept() {
        let mut tokens = vec![Interval(2)];
        mark_positions(&mut tokens, &HashSet::from([0]), Some(0));
        assert_eq!(tokens, vec![Interval(2)]);
    }

    #[test]
    fn every_first_is_a_position() {
        let mut tokens = vec![Interval(1)];
        mark_positions(&mut tokens, &HashSet::from([0]), Some(0));
        assert_eq!(tokens, vec![OrdinalPosition(1)]);
    }

    #[test]
    fn run_stops_at_other_tokens() {
        let mut tokens = vec![
            Interval(2),
            OrdinalPosition(1),
            Weekday(crate::token::Weekday::Monday),
            Interval(3),
        ];
        mark_positions(&mut tokens, &HashSet::from([3]), None);
        assert_eq!(
            tokens,
            vec![
                Interval(2),
                OrdinalPosition(1),
                Weekday(crate::token::Weekday::Monday),
                OrdinalPosition(3),
            ]
        );
    }
}
