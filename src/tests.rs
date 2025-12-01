use crate::{ContextHandler, ContextInterface, Mutex};
use general_mutex::setup_tests;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Level {
    Level0 = 0,
    Level1,
}

impl From<Level> for usize {
    fn from(value: Level) -> Self {
        value as usize
    }
}

struct Handler {}
impl ContextInterface<Level> for Handler {
    fn get_current_level() -> Level {
        Level::Level0
    }
}

const LEVEL0: usize = Level::Level0 as usize;
const LEVEL1: usize = Level::Level1 as usize;
const LEVEL_COUNT: usize = Level::Level1 as usize + 1;

setup_tests!(Mutex<Handler, _, Level, LEVEL0>);

type TestMutex = Mutex<Handler, usize, Level, LEVEL1>;
type TestHandler = ContextHandler<Handler, Level, LEVEL_COUNT>;

#[test]
#[should_panic]
fn try_lock_from_incorrect_level() {
    let mutex: TestMutex = Mutex::new(0);
    mutex.lock(|_| {})
}

#[test]
#[should_panic]
fn try_mut_lock_from_incorrect_level() {
    let mutex: TestMutex = Mutex::new(0);
    mutex.lock(|_| {})
}

#[test]
fn create_context_handler() {
    let _context_handler: TestHandler = ContextHandler::new();
}

#[test]
fn get_context_lock() {
    let context_handler: TestHandler = ContextHandler::new();
    let _lock = context_handler.lock().unwrap();
}

#[test]
fn try_multiple_context_lock() {
    let context_handler: TestHandler = ContextHandler::new();
    let _lock1 = context_handler.lock().unwrap();

    let lock2 = context_handler.lock();
    assert!(lock2.is_none());
}

#[test]
fn drop_lock() {
    let context_handler: TestHandler = ContextHandler::new();

    {
        let _lock1 = context_handler.lock().unwrap();
    }

    let lock2 = context_handler.lock();
    assert!(lock2.is_some());
}

#[test]
fn verify_status() {
    let context_handler: TestHandler = ContextHandler::new();
    let _lock1 = context_handler.lock().unwrap();
    let _lock2 = context_handler.lock();

    let status = context_handler.get_status();
    assert_eq!(status[0].successful_attempts, 1);
    assert_eq!(status[0].failed_attempts, 1);

    // Make sure it cleared
    let status = context_handler.get_status();
    assert_eq!(status[0].successful_attempts, 0);
    assert_eq!(status[0].failed_attempts, 0);
}
