use std::ffi::CStr;

#[derive(Debug)]
pub struct CircularBuffer<const N: usize> {
    start_ptr: usize,
    end_ptr: usize,
    buffer: [u8; N],
}

impl<const N: usize> CircularBuffer<N> {
    pub fn new() -> Self {
        Self {
            start_ptr: 0, // always points to null byte before the next string starts
            end_ptr: 0,
            buffer: [0_u8; N],
        }
    }

    pub fn write_str(&mut self, str: &CStr) {
        // ensure the string can fit in the buffer at all (with null terminator start and end)
        assert!(str.to_str().unwrap().len() + 1 < N);

        // start at the end ptr and write bytes wrapping around
        for b in str.to_bytes_with_nul() {
            self.inc_end_ptr();
            self.buffer[self.end_ptr] = *b;

            // if the start and and pointers collide, fast forward the start_ptr to the next char
            // after it encounters a nul terminator
            if self.end_ptr == self.start_ptr {
            	self.fastforward_start_ptr();
            }
        }
    }

    fn inc_start_ptr(&mut self) {
        self.start_ptr = (self.start_ptr + 1) % N;
    }

    fn inc_end_ptr(&mut self) {
        self.end_ptr = (self.end_ptr + 1) % N;
    }

    fn fastforward_start_ptr(&mut self) {
        println!("Fastforwarding!!");
        while self.buffer[self.start_ptr] != b'\0' {
            self.inc_start_ptr();
        }
    }
}

impl<const N: usize> IntoIterator for CircularBuffer<N> {
    type Item = String;
    type IntoIter = CircularBufferIterator<N>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        let cursor = self.start_ptr;
        Self::IntoIter {
            buffer: Box::new(self),
            cursor,
        }
    }
}

pub struct CircularBufferIterator<const N: usize> {
    buffer: Box<CircularBuffer<N>>,
    cursor: usize,
}

impl<const N: usize> Iterator for CircularBufferIterator<N> {
    type Item = String;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.cursor == self.buffer.end_ptr {
            None
        } else {
            //
            self.cursor = (self.cursor + 1) % N;
            // copy into an intermediate so we can return an owned string
            let mut s: Vec<u8> = Vec::new();
            while self.buffer.buffer[self.cursor] != b'\0' {
                s.push(self.buffer.buffer[self.cursor]);
                self.cursor = (self.cursor + 1) % N;
            }
            Some(String::from_utf8(s).unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn is_initially_empty() {
        let cb: CircularBuffer<100> = CircularBuffer::new();

        assert_eq!(cb.start_ptr, 0);
        assert_eq!(cb.end_ptr, 0);
        assert_eq!(
            cb.into_iter().collect::<Vec<String>>(),
            Vec::<String>::new()
        )
    }

    #[test]
    fn can_write_one_string_no_wrap() {
        let mut cb: CircularBuffer<100> = CircularBuffer::new();

        cb.write_str(&CString::new("hello").unwrap());

        assert_eq!(cb.start_ptr, 0);
        assert_eq!(cb.end_ptr, 6);
        assert_eq!(
            cb.into_iter().collect::<Vec<_>>(),
            vec![String::from("hello")]
        )
    }

    #[test]
    fn can_write_two_strings_no_wrap() {
        let mut cb: CircularBuffer<100> = CircularBuffer::new();

        cb.write_str(&CString::new("hello").unwrap());
        println!("{:?}", cb);
        assert_eq!(cb.start_ptr, 0);
        assert_eq!(cb.end_ptr, 6);

        cb.write_str(&CString::new("world").unwrap());
        println!("{:?}", cb);
        assert_eq!(cb.start_ptr, 0);
        assert_eq!(cb.end_ptr, 12);

        assert_eq!(
            cb.into_iter().collect::<Vec<_>>(),
            vec![String::from("hello"), String::from("world")]
        )
    }

    #[test]
    fn can_write_two_strings_with_wrap() {
        let mut cb: CircularBuffer<10> = CircularBuffer::new();

        cb.write_str(&CString::new("hello").unwrap());
        println!("{:?}", cb);
        assert_eq!(cb.start_ptr, 0);
        assert_eq!(cb.end_ptr, 6);
        
        cb.write_str(&CString::new("world").unwrap());
        println!("{:?}", cb);
        assert_eq!(cb.start_ptr, 6);
        assert_eq!(cb.end_ptr, 2);

        assert_eq!(
            cb.into_iter().collect::<Vec<_>>(),
            vec![String::from("world")]
        )
    }

    #[test]
    fn can_write_three_strings_with_wrap() {
        let mut cb: CircularBuffer<10> = CircularBuffer::new();

        cb.write_str(&CString::new("hello").unwrap());        
        cb.write_str(&CString::new("world").unwrap());
        cb.write_str(&CString::new("peace").unwrap());

        assert_eq!(
            cb.into_iter().collect::<Vec<_>>(),
            vec![String::from("peace")]
        )
    }

    #[test]
    fn can_write_three_strings_with_partial_wrap() {
        let mut cb: CircularBuffer<15> = CircularBuffer::new();

        cb.write_str(&CString::new("hello").unwrap());        
        cb.write_str(&CString::new("world").unwrap());
        cb.write_str(&CString::new("peace").unwrap());

        assert_eq!(
            cb.into_iter().collect::<Vec<_>>(),
            vec![String::from("world"), String::from("peace")]
        )
    }

    #[test]
    fn can_write_one_string_fitting_buffer() {
        // it requires two null bytes to work so len(string) + 2
        let mut cb: CircularBuffer<7> = CircularBuffer::new();

        cb.write_str(&CString::new("hello").unwrap());        

        assert_eq!(
            cb.into_iter().collect::<Vec<_>>(),
            vec![String::from("hello")]
        )
    }
}
