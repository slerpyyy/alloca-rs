#[test]
fn create_simple() {
    let x = crate::with_raw(4096, |_| 42);
    assert_eq!(x, 42);
}

#[test]
fn write_simple() {
    let x = unsafe {
        crate::with_slice_assume_init(4096, |memory| {
            memory[0] = 42;
            memory[1] = 3;
            memory[3072] = 4;

            assert_eq!(memory[0], 42);
            assert_eq!(memory[1], 3);
            assert_eq!(memory[3072], 4);

            memory[0] + memory[1] + memory[3072]
        })
    };

    assert_eq!(x, 42 + 3 + 4);
}

#[test]
fn with_f32_simple() {
    unsafe {
        crate::with_assume_init(|data| {
            *data = 2.0;
            assert_eq!(data, &2.0);
        });
    }
}

#[test]
#[should_panic(expected = "Hello!")]
fn propergate_panic() {
    crate::with_bytes(32, |_| {
        panic!("Hello!");
    });
}
