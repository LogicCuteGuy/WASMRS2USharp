//! Test for the simple multi-behavior example

use crate::*;

#[test]
fn test_counter_creation() {
    let counter = Counter {
        current_count: 0,
        max_count: 100,
        display: None,
    };
    
    assert_eq!(counter.current_count, 0);
    assert_eq!(counter.max_count, 100);
}

#[test]
fn test_display_creation() {
    let display = Display {
        count_text: None,
        status_text: None,
        counter: None,
    };
    
    assert!(display.count_text.is_none());
    assert!(display.status_text.is_none());
    assert!(display.counter.is_none());
}

#[test]
fn test_shared_functions() {
    // Test the shared utility functions
    assert_eq!(get_count_status(0, 100), "Ready to start counting!");
    assert_eq!(get_count_status(100, 100), "Maximum count reached!");
    assert_eq!(get_count_status(50, 100), "Halfway there!");
    
    assert_eq!(format_number(500), "500");
    assert_eq!(format_number(1500), "1,000+");
}

#[test]
fn test_counter_logic() {
    let mut counter = Counter {
        current_count: 0,
        max_count: 10,
        display: None,
    };
    
    // Test increment
    counter.increment();
    assert_eq!(counter.current_count, 1);
    
    // Test decrement
    counter.decrement();
    assert_eq!(counter.current_count, 0);
    
    // Test reset
    counter.current_count = 5;
    counter.reset();
    assert_eq!(counter.current_count, 0);
}

#[test]
fn test_counter_bounds() {
    let mut counter = Counter {
        current_count: 0,
        max_count: 2,
        display: None,
    };
    
    // Test max bound
    counter.increment(); // 1
    counter.increment(); // 2
    counter.increment(); // Should stay at 2
    assert_eq!(counter.current_count, 2);
    
    // Test min bound
    counter.decrement(); // 1
    counter.decrement(); // 0
    counter.decrement(); // Should stay at 0
    assert_eq!(counter.current_count, 0);
}