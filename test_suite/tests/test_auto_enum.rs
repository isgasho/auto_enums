#![cfg_attr(feature = "unstable", feature(proc_macro_hygiene))]
#![cfg_attr(feature = "unstable", feature(stmt_expr_attributes))]
#![cfg_attr(feature = "unstable", feature(arbitrary_self_types, futures_api, pin))]
#![cfg_attr(feature = "unstable", feature(fn_traits, unboxed_closures))]
#![cfg_attr(feature = "unstable", feature(read_initializer))]
#![cfg_attr(feature = "unstable", feature(trusted_len))]
#![cfg_attr(feature = "unstable", feature(exact_size_is_empty))]
#![cfg_attr(feature = "unstable", feature(try_trait))]
#![cfg_attr(feature = "unstable", feature(unsized_locals))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(not(feature = "std"), feature = "unstable"), feature(alloc))]
#![allow(unused_imports)]
#![cfg(test)]

#[cfg(all(not(feature = "std"), feature = "unstable"))]
#[macro_use]
extern crate alloc;
#[cfg(feature = "std")]
extern crate core;

#[macro_use]
extern crate auto_enumerate;

#[test]
fn stable_1_30() {
    const ANS: &[i32] = &[28, 3];

    #[auto_enum(Iterator)]
    fn match_(x: usize) -> impl Iterator<Item = i32> {
        match x {
            0 => 1..8,
            n if n > 3 => 2..=10,
            _ => (0..2).map(|x| x + 1),
        }
    }
    for i in 0..2 {
        assert_eq!(match_(i).fold(0, |sum, x| sum + x), ANS[i]);
    }

    #[cfg_attr(feature = "rustfmt", rustfmt_skip)]
    #[allow(unused_unsafe)]
    #[auto_enum(Iterator)]
    fn block(x: usize) -> impl Iterator<Item = i32> {
        {{{ unsafe {{{ unsafe { unsafe {{
            match x {
                0 => 1..8,
                n if n > 3 => 2..=10,
                _ => (0..2).map(|x| x + 1),
            }
        }}}}}}}}}
    }
    for i in 0..2 {
        assert_eq!(block(i).fold(0, |sum, x| sum + x), ANS[i]);
    }

    #[auto_enum(Iterator)]
    fn if_(x: usize) -> impl Iterator<Item = i32> {
        if x == 0 {
            1..8
        } else if x > 3 {
            2..=10
        } else {
            (0..2).map(|x| x + 1)
        }
    }
    for i in 0..2 {
        assert_eq!(if_(i).fold(0, |sum, x| sum + x), ANS[i]);
    }

    #[auto_enum(Iterator)]
    fn method_call(x: usize) -> impl Iterator<Item = i32> {
        if x == 0 {
            1..8
        } else if x > 3 {
            2..=10
        } else {
            (0..2).map(|x| x + 1)
        }
        .map(|x| x + 1)
        .map(|x| x - 1)
    }
    for i in 0..2 {
        assert_eq!(method_call(i).fold(0, |sum, x| sum + x), ANS[i]);
    }

    #[auto_enum(Iterator)]
    fn no_return(x: usize) -> impl Iterator<Item = i32> {
        match x {
            0 => 1..8,
            3 => panic!(),
            _ => (0..2).map(|x| x + 1),
        }
    }
    for i in 0..2 {
        assert_eq!(no_return(i).fold(0, |sum, x| sum + x), ANS[i]);
    }

    #[auto_enum(Iterator)]
    fn no_return2(x: usize) -> impl Iterator<Item = i32> {
        match x {
            0 => 1..8,
            3 => match x {
                0 => panic!(),
                1..=3 => panic!(),
                _ => unreachable!(),
            },
            _ => (0..2).map(|x| x + 1),
        }
    }
    for i in 0..2 {
        assert_eq!(no_return2(i).fold(0, |sum, x| sum + x), ANS[i]);
    }

    #[auto_enum(Iterator)]
    fn no_return3(x: usize) -> impl Iterator<Item = i32> {
        match x {
            0 => 1..8,
            3 => match x {
                0 => panic!(),
                1..=3 => (1..4).map(|x| x + 1),
                _ => unreachable!(),
            },
            _ => (0..2).map(|x| x + 1),
        }
    }
    for i in 0..2 {
        assert_eq!(no_return3(i).fold(0, |sum, x| sum + x), ANS[i]);
    }
    #[auto_enum(Iterator)]
    fn no_return4(x: usize) -> impl Iterator<Item = i32> {
        if x == 0 {
            1..8
        } else if x > 3 {
            panic!();
        } else {
            (0..2).map(|x| x + 1)
        }
    }
    for i in 0..2 {
        assert_eq!(no_return4(i).fold(0, |sum, x| sum + x), ANS[i]);
    }

    #[auto_enum(Iterator)]
    fn marker1(x: usize) -> impl Iterator<Item = i32> {
        if x > 10 {
            return marker!((0..x as _).map(|x| x - 1));
        }
        if x == 0 {
            1..8
        } else if x > 3 {
            2..=10
        } else {
            (0..2).map(|x| x + 1)
        }
    }
    for i in 0..2 {
        assert_eq!(marker1(i).fold(0, |sum, x| sum + x), ANS[i]);
    }

    #[auto_enum(Iterator)]
    fn marker_in_loop(mut x: i32) -> impl Iterator<Item = i32> {
        loop {
            if x < 0 {
                break marker!(x..0);
            } else if x % 5 == 0 {
                break marker!(0..=x);
            }
            x -= 1;
        }
    }
    assert_eq!(marker_in_loop(14).fold(0, |sum, x| sum + x), 55);
    assert_eq!(marker_in_loop(-5).fold(0, |sum, x| sum + x), -15);

    #[auto_enum(marker(marker_a), Iterator)]
    fn marker2(x: i32, y: i32) -> impl Iterator<Item = i32> {
        #[auto_enum(Iterator)]
        let iter = match x {
            0 => 2..8,
            _ if y < 0 => return marker_a!(y..=0),
            _ => vec![2, 0].into_iter(),
        };

        match y {
            0 => iter.flat_map(|x| 0..x),
            _ => iter.map(|x| x + 1),
        }
    }
    assert_eq!(marker2(10, 10).fold(0, |sum, x| sum + x), 4);

    #[auto_enum(Iterator)]
    fn marker3(x: i32, y: i32) -> impl Iterator<Item = i32> {
        let iter;
        #[auto_enum(Iterator)]
        match x {
            0 => iter = marker!(2..8),
            _ => iter = marker!(vec![2, 0].into_iter()),
        };

        if y < 0 {
            return marker!(y..=0);
        }
        match y {
            0 => iter.flat_map(|x| 0..x),
            _ => iter.map(|x| x + 1),
        }
    }
    assert_eq!(marker3(10, 10).fold(0, |sum, x| sum + x), 4);

    #[auto_enum(marker(marker_a), Iterator)]
    fn marker4(x: i32, y: i32) -> impl Iterator<Item = i32> {
        let iter;
        #[auto_enum(Iterator)]
        match x {
            0 => iter = marker!(2..8),
            _ if y < 0 => return marker_a!(y..=0),
            _ => iter = marker!(vec![2, 0].into_iter()),
        };

        match y {
            0 => iter.flat_map(|x| 0..x),
            _ => iter.map(|x| x + 1),
        }
    }
    assert_eq!(marker4(10, 10).fold(0, |sum, x| sum + x), 4);

    #[auto_enum(Iterator)]
    fn rec_match_in_match(x: usize) -> impl Iterator<Item = i32> {
        match x {
            0 => 1..8,
            #[rec]
            n if n > 3 => match x {
                2..=10 => (1..x as _).map(|x| x - 1),
                _ => 2..=10,
            },
            _ => (0..2).map(|x| x + 1),
        }
    }
    for i in 0..2 {
        assert_eq!(rec_match_in_match(i).fold(0, |sum, x| sum + x), ANS[i]);
    }

    #[cfg_attr(feature = "rustfmt", rustfmt_skip)]
    #[allow(unused_unsafe)]
    #[auto_enum(Iterator)]
    fn rec_in_block(x: usize) -> impl Iterator<Item = i32> {
        {{{ unsafe {{{ unsafe { unsafe {{
            match x {
                0 => 1..8,
                #[rec]
                n if n > 3 => {{{ unsafe {{
                    if x > 10 {
                        (-10..=x as _).map(|x| x - 4)
                    } else {
                        (1..=4).map(|x| x - 4)
                    }
                }}}}}
                _ => (0..2).map(|x| x + 1),
            }
        }}}}}}}}}
    }
    for i in 0..2 {
        assert_eq!(rec_in_block(i).fold(0, |sum, x| sum + x), ANS[i]);
    }

    #[auto_enum(Iterator)]
    fn rec_match_in_if(x: usize) -> impl Iterator<Item = i32> {
        if x == 0 {
            1..8
        } else if x > 3 {
            #[rec]
            match x {
                1..=4 => 2..=10,
                _ => (11..20).map(|x| x - 1),
            }
        } else {
            (0..2).map(|x| x + 1)
        }
    }
    for i in 0..2 {
        assert_eq!(rec_match_in_if(i).fold(0, |sum, x| sum + x), ANS[i]);
    }

    #[auto_enum(Iterator)]
    fn rec_if_in_if(x: usize) -> impl Iterator<Item = i32> {
        if x == 0 {
            1..8
        } else if x > 3 {
            #[rec]
            {
                if x > 4 {
                    2..=10
                } else {
                    (11..20).map(|x| x - 1)
                }
            }
        } else {
            (0..2).map(|x| x + 1)
        }
    }
    for i in 0..2 {
        assert_eq!(rec_if_in_if(i).fold(0, |sum, x| sum + x), ANS[i]);
    }

    #[auto_enum(Iterator)]
    fn rec_nop(x: usize) -> impl Iterator<Item = i32> {
        if x == 0 {
            1..8
        } else if x > 3 {
            #[rec]
            2..=10
        } else {
            (0..2).map(|x| x + 1)
        }
    }
    for i in 0..2 {
        assert_eq!(rec_nop(i).fold(0, |sum, x| sum + x), ANS[i]);
    }

    #[auto_enum(Iterator)]
    fn rec_no_return(x: usize) -> impl Iterator<Item = i32> {
        match x {
            0 => 1..8,
            #[rec]
            3 => panic!(),
            _ => (0..2).map(|x| x + 1),
        }
    }
    for i in 0..2 {
        assert_eq!(rec_no_return(i).fold(0, |sum, x| sum + x), ANS[i]);
    }

    #[auto_enum(Transpose, Iterator, Clone)]
    fn transpose(x: isize) -> Option<impl Iterator<Item = i32> + Clone> {
        match x {
            0 => Some(1..8),
            _ if x < 0 => return None,
            _ => Some(0..=10),
        }
        .transpose()
        .map(|i| i.map(|x| x + 1).map(|x| x - 1))
    }
    assert_eq!(transpose(0).unwrap().fold(0, |sum, x| sum + x), 28);
}

#[cfg(feature = "std")]
#[test]
fn stable_1_30_std() {
    use std::{error::Error, fs, io, path::Path};

    #[auto_enum(Transpose, Write)]
    fn transpose_ok(file: Option<&Path>) -> io::Result<impl io::Write> {
        if let Some(file) = file {
            fs::File::create(file)
        } else {
            Ok(io::stdout())
        }
        .transpose_ok()
    }
    assert!(transpose_ok(None).is_ok());

    #[auto_enum(Transpose, Write)]
    fn transpose_option(file: Option<&Path>) -> Option<impl io::Write> {
        if let Some(file) = file {
            fs::File::create(file).ok()
        } else {
            Some(io::stdout())
        }
        .transpose()
    }
    assert!(transpose_option(None).is_some());

    #[enum_derive(Debug, Display, Error)]
    enum IoError {
        Io(io::Error),
        Io2(io::Error),
    }

    #[auto_enum(Transpose, Write, Debug, Display, Error)]
    fn transpose_result(file: Option<&Path>) -> Result<impl io::Write, impl Error> {
        if let Some(file) = file {
            fs::File::create(file).map_err(IoError::Io)
        } else {
            let out: Result<io::Stdout, io::Error> = Ok(io::stdout());
            out
        }
        .transpose()
    }
    assert!(transpose_result(None).is_ok());

    #[auto_enum(Transpose, Debug, Display, Error)]
    fn transpose_err(file: Option<&Path>) -> Result<(), impl Error> {
        if let Some(_) = file {
            Err(io::Error::from(io::ErrorKind::NotFound)).map_err(IoError::Io)
        } else {
            Err(io::Error::from(io::ErrorKind::NotFound))
        }
        .transpose_err()
    }
    assert!(transpose_err(None).is_err());
}

#[cfg(feature = "unstable")]
#[test]
fn nightly() {
    const ANS: &[i32] = &[28, 3];

    // let match
    for i in 0..2 {
        #[auto_enum(Iterator)]
        let iter = match i {
            0 => 1..8,
            _ => vec![1, 2, 0].into_iter(),
        };
        assert_eq!(iter.fold(0, |sum, x| sum + x), ANS[i]);
    }

    // let if
    for i in 0..2 {
        #[auto_enum(Iterator)]
        let iter = if i == 0 {
            1..8
        } else if i > 3 {
            1..=10
        } else {
            vec![1, 2, 0].into_iter()
        };
        assert_eq!(iter.fold(0, |sum, x| sum + x), ANS[i]);
    }

    // no return
    for i in 0..2 {
        #[auto_enum(Iterator)]
        let iter = match i {
            0 => 1..8,
            #[never]
            5..=10 => loop {
                panic!()
            },
            _ => vec![1, 2, 0].into_iter(),
        };
        assert_eq!(iter.fold(0, |sum, x| sum + x), ANS[i]);
    }
    for i in 0..2 {
        #[auto_enum(Iterator)]
        let iter = match i {
            0 => 1..8,
            5..=10 => panic!(),
            11..=20 => unreachable!(),
            21..=30 => break,
            31..=40 => continue,
            41..=50 => return,
            _ => vec![1, 2, 0].into_iter(),
        };
        assert_eq!(iter.fold(0, |sum, x| sum + x), ANS[i]);
    }
    for i in 0..2 {
        #[auto_enum(Iterator)]
        let iter = if i > 3 {
            #[never]
            loop {
                panic!()
            }
        } else if i == 0 {
            1..8
        } else {
            vec![1, 2, 0].into_iter()
        };
        assert_eq!(iter.fold(0, |sum, x| sum + x), ANS[i]);
    }

    // rec
    for i in 0..2 {
        #[auto_enum(Iterator)]
        let iter = if i > 3 {
            #[rec]
            match i {
                1..=10 => (1..3).map(|x| x + 1),
                11..=20 => 4..=5,
                _ => (5..10).map(|x| x - 1),
            }
        } else if i == 0 {
            1..8
        } else {
            vec![1, 2, 0].into_iter()
        };
        assert_eq!(iter.fold(0, |sum, x| sum + x), ANS[i]);
    }

    // never opt
    for i in 0..2 {
        #[auto_enum(never, Iterator)]
        let iter = match i {
            0 => marker!(1..8),
            5..=10 => loop {
                panic!()
            },
            _ => marker!(vec![1, 2, 0].into_iter()),
        };
        assert_eq!(iter.fold(0, |sum, x| sum + x), ANS[i]);
    }
    for i in 0..2 {
        #[cfg_attr(feature = "rustfmt", rustfmt_skip)]
        #[auto_enum(Iterator)]
        let iter = match i {
            0 => 1..8,
            #[never]
            5..=10 => loop {
                panic!()
            },
            _ => {
                match i {
                    #[never]
                    5..=10 => loop {
                        panic!()
                    },
                    #[never]
                    11..=20 => loop {
                        panic!()
                    },
                    _ => vec![1, 2, 0].into_iter(),
                }
            }
        };
        assert_eq!(iter.fold(0, |sum, x| sum + x), ANS[i]);
    }
    for i in 0..2 {
        #[cfg_attr(feature = "rustfmt", rustfmt_skip)]
        #[auto_enum(Iterator)]
        let iter = match i {
            0 => 1..8,
            #[never]
            5..=10 => loop {
                panic!()
            },
            1..=4 => vec![1, 2, 0].into_iter(),
            _ => {
                match i {
                    #[never]
                    5..=10 => loop {
                        panic!()
                    },
                    #[never]
                    11..=20 => loop {
                        panic!()
                    },
                    _ => panic!(),
                }
            }
        };
        assert_eq!(iter.fold(0, |sum, x| sum + x), ANS[i]);
    }

    fn manual_4(x: usize) -> impl Iterator<Item = i32> + Clone {
        #[auto_enum(Iterator, Clone)]
        {
            if x == 0 {
                return marker!(2..8);
            }
            match x {
                _ if x < 2 => vec![2, 0].into_iter(),
                _ => 2..=10,
            }
        }
    }
    for i in 0..2 {
        assert_eq!(manual_4(i).clone().fold(0, |sum, x| sum + x), ANS[i] - 1);
    }

    fn manual_5(x: usize) -> impl Iterator<Item = i32> + Clone {
        #[auto_enum(Iterator, Clone)]
        (0..x as i32).map(|x| x + 1).flat_map(|x| {
            if x > 10 {
                marker!(0..x)
            } else {
                marker!(-100..=0)
            }
        })
    }
    for i in 0..2 {
        let _ = manual_5(i).fold(0, |sum, x| sum + x);
    }

    fn manual_6(x: usize) -> impl Iterator<Item = i32> + Clone {
        let a;

        #[auto_enum(Iterator, Clone)]
        match x {
            0 => a = marker!(2..8),
            _ if x < 2 => a = marker!(vec![2, 0].into_iter()),
            _ => a = marker!(2..=10),
        };
        a
    }
    for i in 0..2 {
        assert_eq!(manual_6(i).fold(0, |sum, x| sum + x), ANS[i] - 1);
    }

    /*
    This can not be supported. It is parsed as follows.
        expected: ExprAssign { left: ExprPath, right: ExprMatch, .. }
           found: ExprPath
    #[auto_enum(Iterator, Clone)]
    a = match x {
        0 => 2..8,
        _ if x < 2 => vec![2, 0].into_iter(),
        _ => 2..=10,
    };
    */
    fn manual_7(x: usize) -> impl Iterator<Item = i32> + Clone {
        let a;
        a = #[auto_enum(Iterator, Clone)]
        match x {
            0 => 2..8,
            _ if x < 2 => vec![2, 0].into_iter(),
            _ => 2..=10,
        };
        a
    }
    for i in 0..2 {
        assert_eq!(manual_7(i).fold(0, |sum, x| sum + x), ANS[i] - 1);
    }

    #[auto_enum(Fn)]
    fn fn_traits(option: bool) -> impl Fn(i32) -> i32 {
        if option {
            |x| x + 1
        } else {
            |y| y - 1
        }
    }
    assert_eq!(fn_traits(true)(1), 2);
}