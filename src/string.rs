use core::slice;

//Reusable type for chars, in xv6 they appear to be
//signed chars by default
#[allow(non_camel_case_types)]
pub type c_char = i8;

#[no_mangle]
pub extern "C" fn strlen(s: *const c_char) -> isize {
    unsafe {
        let mut n = 0;
        while *s.offset(n) != 0 {
            n += 1;
        }
        n
    }
}

#[no_mangle]
pub extern "C" fn strncmp(s: *const c_char, t: *const c_char, n: usize) -> isize {
    //This will give weird results if you pass in a negative number
    let s_slice = unsafe { slice::from_raw_parts(s, n) };
    let t_slice = unsafe { slice::from_raw_parts(t, n) };
    //Iterate though elements of both lists at the same time
    //until we reach a 0
    for (l, r) in s_slice
        .iter()
        //Keep going until we reach the end of s
        .take_while(|&&x| x != 0)
        //Add a null character, take_while will remove the final null character
        //otherwise
        .chain([0].iter())
        .zip(t_slice)
    {
        //Return the difference with unmatching characters
        if l != r {
            return *l as isize - *r as isize;
        }
    }
    0
}
#[no_mangle]
pub extern "C" fn strncpy(s: *mut c_char, t: *const c_char, n: isize) -> *const c_char {
    if n <= 0 {
        return s;
    }

    //Turn both char *'s into slices
    let s_slice = unsafe { slice::from_raw_parts_mut(s, n as usize) };
    let t_slice = unsafe { slice::from_raw_parts(t, n as usize) };
    //Iterate through both slices, copying from t to s
    for (empty, ch) in s_slice.iter_mut().zip(t_slice.iter()) {
        *empty = *ch;
    }

    return s;
}
#[no_mangle]
pub extern "C" fn safestrcpy(s: *mut c_char, t: *const c_char, n: isize) -> *const c_char {
    if n <= 0 {
        return s;
    }
    //Turn both char *'s into slices
    //t_slice is 1 element shorter so we can guarantee there's a zero at
    //the end
    let s_slice = unsafe { slice::from_raw_parts_mut(s, n as usize) };
    let t_slice = unsafe { slice::from_raw_parts(t, (n - 1) as usize) };
    //Iterate through both slices, copying from t to s
    //There's an extra zero at the end of t_slice to guarantee that it is nul-
    //terminated
    for (empty, ch) in s_slice.iter_mut().zip(t_slice.iter().chain([0].iter())) {
        *empty = *ch;
    }

    return s;
}

#[cfg(test)]
mod str_test {
    use proptest::prelude::*;
    use std::cmp;
    use std::ffi::{CStr, CString};
    use std::vec::Vec;
    use string::{c_char, safestrcpy, strncmp, strncpy};
    proptest! {

        #[test]
        fn strncmp_pseudo_symmetric(s in "[[:print:]]*", t in "[[:print:]]*") {
            let max_length = cmp::max(s.len(),t.len())+1;
            let s = CString::new(s).unwrap().as_ptr();
            let t = CString::new(t).unwrap().as_ptr();
            assert_eq!(strncmp(s,t,max_length),-1*strncmp(t,s,max_length))
        }
        #[test]
        fn strncmp_reflexive(s in "[[:print:]]*") {
            let len = s.len()+1;
            let s = CString::new(s).unwrap().as_ptr();
            assert_eq!(strncmp(s,s,len), 0)
        }
        #[should_panic]
        #[test]
        fn strncmp_not_equal(s in "[[:print:]]*", t in "[[:print:]]*") {
            let len = s.len()+1;
            let s = CString::new(s).unwrap().as_ptr();
            let t = CString::new(t).unwrap().as_ptr();
            if t != s {
                assert_eq!(strncmp(s,t,len), 0)
            } else {
                panic!()
            }
        }
        #[test]
        fn strncpy_equals(s in "[[:print:]]{0,255}") {
            let len = s.len() + 1;
            let from_str = CString::new(s.clone()).unwrap();
            let mut buffer: [i8; 256] = [0; 256];

            //Check to make sure the string has the right length
            assert_eq!(from_str.as_bytes_with_nul().len(),len);

            //Make sure we get the same pointer back
            assert_eq!(
                strncpy(buffer.as_mut_ptr(),from_str.as_ptr(),len as isize),
                    buffer.as_ptr());


            let final_bytes: Vec<u8> = buffer
                .iter()
                .map(|&x| x as u8)
                .take_while(|&x| x != 0)
                .collect();

            let final_str = CString::new(final_bytes).unwrap();
            assert_eq!(final_str.as_bytes_with_nul().len(),len);
            assert_eq!(final_str,from_str);
        }
        #[test]
        fn safestrcpy_equals(s in "[[:print:]]{0,255}") {
            let len = s.len() + 1;
            let from_str = CString::new(s.clone()).unwrap();
            let mut buffer: [i8; 256] = [0; 256];

            //Check to make sure the string has the right length
            assert_eq!(from_str.as_bytes_with_nul().len(),len);

            //Make sure we get the same pointer back
            assert_eq!(
                safestrcpy(buffer.as_mut_ptr(),from_str.as_ptr(),len as isize),
                    buffer.as_ptr());


            let final_bytes: Vec<u8> = buffer
                .iter()
                .map(|&x| x as u8)
                .take_while(|&x| x != 0)
                .collect();

            let final_str = CString::new(final_bytes).unwrap();
            assert_eq!(final_str.as_bytes_with_nul().len(),len);
            assert_eq!(final_str,from_str);
        }
    }
}
