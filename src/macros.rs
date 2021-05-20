macro_rules! all_not_equal {
    () => {};
    ($first:expr) => {};
    ($first:expr, $($vals:expr),+) => {{
        {
            $(
                assert_ne!($first, $vals);
            )*
            all_not_equal!($($vals),+);
        }
    }};
}

macro_rules! all_less_than {
    ($val:expr, $($rest:expr),*) => {{
        {
            $(
                assert!($rest < $val);
            )*
        }
    }}
}

macro_rules! mut_index {
    ($arr:expr, $idx:expr) => {{
        {
            println!("{:?}", $arr[$idx])
        }
    }};

    ($arr:expr, $($idxs:expr),*) => {{
        {
            all_not_equal!($($idxs),*);
            all_less_than!(($arr).len(), $($idxs),*);
            unsafe {
                ( $(&mut *(($arr).get_unchecked_mut($idxs) as *mut _), )* )
            }
        }
    }};
}