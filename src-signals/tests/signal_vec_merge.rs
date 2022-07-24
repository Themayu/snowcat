mod util;

use crate::util::{assert_signal_vec_eq, with_noop_context};
use futures_signals::signal_vec::{MutableVec, SignalVecExt, VecDiff};
use snowcat_signals::signal_vec::merge::MergedVecItem;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::rc::Rc;
use std::task::Poll;

const RNG_SEED: u64 = 0;

#[cfg(test)]
#[ctor::ctor]
fn init() {
	use stderrlog::LogLevelNum;

	let mut logger = stderrlog::new();
	logger.module(module_path!());

	if let Some(_) = option_env!("LOG_SIGNALS") {
		logger.module("snowcat_signals::signal_vec::merge");
	}

	match option_env!("LOG_LEVEL").map(|opt| opt.to_ascii_lowercase()).as_deref() {
		Some("e")
		| Some("error")
		| Some("errors") => logger.verbosity(LogLevelNum::Error),

		Some("w")
		| Some("warn")
		| Some("warnings") => logger.verbosity(LogLevelNum::Warn),

		Some("i")
		| Some("info") => logger.verbosity(LogLevelNum::Info),

		Some("d")
		| Some("debug") => logger.verbosity(LogLevelNum::Debug),

		Some("t")
		| Some("trace") => logger.verbosity(LogLevelNum::Trace),

		// don't enable the logger
		_ => logger.verbosity(LogLevelNum::Off),
	};

	logger.init()
		.unwrap();
}

#[test]
fn merge_replace_less() {
	dataset::seed(RNG_SEED);
	let (messages, notifications) = merge_replace_util::initial_dataset();

	let merged = merge_util::signal_from_initial(
		messages.to_vec(),
		notifications.to_vec(),

		|message, notification| {
			if message.timestamp < notification.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	assert_signal_vec_eq(merged, vec![
		Poll::Ready(Some(VecDiff::Replace {
			values: messages.iter().map(to_left).collect()
		})),

		Poll::Ready(Some(VecDiff::Replace {
			values: vec![
				to_left(&messages[0]),
				to_right(&notifications[0]),
				to_left(&messages[1]),
				to_left(&messages[2]),
				to_left(&messages[3]),
				to_right(&notifications[1]),
				to_right(&notifications[2]),
				to_right(&notifications[3]),
				to_left(&messages[4]),
				to_right(&notifications[4]),
				to_left(&messages[5]),
				to_left(&messages[6]),
				to_left(&messages[7]),
				to_right(&notifications[5]),
			],
		})),

		Poll::Ready(None),
	]);
}

#[test]
fn merge_replace_less_equal() {
	dataset::seed(RNG_SEED);
	let (messages, notifications) = merge_replace_util::initial_dataset();

	let merged = merge_util::signal_from_initial(
		messages.to_vec(),
		notifications.to_vec(),

		|message, notification| {
			if message.timestamp <= notification.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	assert_signal_vec_eq(merged, vec![
		Poll::Ready(Some(VecDiff::Replace {
			values: messages.iter().map(to_left).collect()
		})),

		Poll::Ready(Some(VecDiff::Replace {
			values: vec![
				to_left(&messages[0]),
				to_right(&notifications[0]),
				to_left(&messages[1]),
				to_left(&messages[2]),
				to_left(&messages[3]),
				to_right(&notifications[1]),
				to_right(&notifications[2]),
				to_right(&notifications[3]),
				to_left(&messages[4]),
				to_right(&notifications[4]),
				to_left(&messages[5]),
				to_left(&messages[6]),
				to_left(&messages[7]),
				to_right(&notifications[5]),
			],
		})),

		Poll::Ready(None),
	]);
}

#[test]
fn merge_replace_greater() {
	dataset::seed(RNG_SEED);
	let (messages, notifications) = merge_replace_util::initial_dataset();

	let merged = merge_util::signal_from_initial(
		messages.to_vec(),
		notifications.to_vec(),

		|message, notification| {
			if message.timestamp > notification.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	assert_signal_vec_eq(merged, vec![
		Poll::Ready(Some(VecDiff::Replace {
			values: messages.iter().map(to_left).collect()
		})),

		Poll::Ready(Some(VecDiff::Replace {
			values: vec![
				to_right(&notifications[0]),
				to_right(&notifications[1]),
				to_right(&notifications[2]),
				to_right(&notifications[3]),
				to_right(&notifications[4]),
				to_right(&notifications[5]),
				to_left(&messages[0]),
				to_left(&messages[1]),
				to_left(&messages[2]),
				to_left(&messages[3]),
				to_left(&messages[4]),
				to_left(&messages[5]),
				to_left(&messages[6]),
				to_left(&messages[7]),
			],
		})),

		Poll::Ready(None),
	]);
}

#[test]
fn merge_replace_greater_equal() {
	dataset::seed(RNG_SEED);
	let (messages, notifications) = merge_replace_util::initial_dataset();

	let merged = merge_util::signal_from_initial(
		messages.to_vec(),
		notifications.to_vec(),

		|message, notification| {
			if message.timestamp >= notification.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	assert_signal_vec_eq(merged, vec![
		Poll::Ready(Some(VecDiff::Replace {
			values: messages.iter().map(to_left).collect()
		})),

		Poll::Ready(Some(VecDiff::Replace {
			values: vec![
				to_right(&notifications[0]),
				to_right(&notifications[1]),
				to_right(&notifications[2]),
				to_right(&notifications[3]),
				to_right(&notifications[4]),
				to_right(&notifications[5]),
				to_left(&messages[0]),
				to_left(&messages[1]),
				to_left(&messages[2]),
				to_left(&messages[3]),
				to_left(&messages[4]),
				to_left(&messages[5]),
				to_left(&messages[6]),
				to_left(&messages[7]),
			],
		})),

		Poll::Ready(None),
	]);
}

#[test]
fn merge_insert_left() {
	dataset::seed(RNG_SEED);
	let messages = dataset::messages();
	let notifications = dataset::notifications();

	let merged = merge_util::signal_from_ops(
		vec![VecDiff::InsertAt { index: 0, value: messages[0].clone() },
		     VecDiff::InsertAt { index: 1, value: messages[1].clone() },
		     VecDiff::InsertAt { index: 2, value: messages[2].clone() },
		     VecDiff::InsertAt { index: 3, value: messages[3].clone() },
		     VecDiff::InsertAt { index: 4, value: messages[4].clone() },
		     VecDiff::InsertAt { index: 5, value: messages[5].clone() },
		     VecDiff::InsertAt { index: 6, value: messages[6].clone() },
		     VecDiff::InsertAt { index: 7, value: messages[7].clone() }],
		vec![VecDiff::Replace { values: notifications.to_vec() }],

		|message, timestamp| {
			if message.timestamp <= timestamp.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	assert_signal_vec_eq(merged, vec![
		Poll::Ready(Some(VecDiff::Replace {
			values: notifications.iter()
				.map(to_right)
				.collect()
		})),

		Poll::Ready(Some(VecDiff::InsertAt { index:  0, value: to_left(&messages[0]) })),
		Poll::Ready(Some(VecDiff::InsertAt { index:  2, value: to_left(&messages[1]) })),
		Poll::Ready(Some(VecDiff::InsertAt { index:  3, value: to_left(&messages[2]) })),
		Poll::Ready(Some(VecDiff::InsertAt { index:  4, value: to_left(&messages[3]) })),
		Poll::Ready(Some(VecDiff::InsertAt { index:  8, value: to_left(&messages[4]) })),
		Poll::Ready(Some(VecDiff::InsertAt { index: 10, value: to_left(&messages[5]) })),
		Poll::Ready(Some(VecDiff::InsertAt { index: 11, value: to_left(&messages[6]) })),
		Poll::Ready(Some(VecDiff::InsertAt { index: 12, value: to_left(&messages[7]) })),

		Poll::Ready(None),
	]);
}

#[test]
fn merge_insert_right() {
	dataset::seed(RNG_SEED);
	let messages = dataset::messages();
	let notifications = dataset::notifications();

	let merged = merge_util::signal_from_ops(
		vec![VecDiff::Replace { values: messages.to_vec() }],
		vec![VecDiff::InsertAt { index: 0, value: notifications[0].clone() },
		     VecDiff::InsertAt { index: 1, value: notifications[1].clone() },
		     VecDiff::InsertAt { index: 2, value: notifications[2].clone() },
		     VecDiff::InsertAt { index: 3, value: notifications[3].clone() },
		     VecDiff::InsertAt { index: 4, value: notifications[4].clone() },
		     VecDiff::InsertAt { index: 5, value: notifications[5].clone() }],

		|message, timestamp| {
			if message.timestamp <= timestamp.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	assert_signal_vec_eq(merged, vec![
		Poll::Ready(Some(VecDiff::Replace {
			values: messages.iter()
				.map(|value| into_left(value.clone()))
				.collect()
		})),

		Poll::Ready(Some(VecDiff::InsertAt { index:  1, value: to_right(&notifications[0]) })),
		Poll::Ready(Some(VecDiff::InsertAt { index:  5, value: to_right(&notifications[1]) })),
		Poll::Ready(Some(VecDiff::InsertAt { index:  6, value: to_right(&notifications[2]) })),
		Poll::Ready(Some(VecDiff::InsertAt { index:  7, value: to_right(&notifications[3]) })),
		Poll::Ready(Some(VecDiff::InsertAt { index:  9, value: to_right(&notifications[4]) })),
		Poll::Ready(Some(VecDiff::InsertAt { index: 13, value: to_right(&notifications[5]) })),

		Poll::Ready(None),
	]);
}

#[test]
fn merge_update_at() {
	use dataset::Message;

	dataset::seed(RNG_SEED);
	let messages = dataset::messages();
	let notifications = dataset::notifications();

	let new_messages = [
		Message::reroll(&messages[2]),
		Message::reroll(&messages[3]),
	];

	let merged = merge_util::signal_from_ops(
		vec![VecDiff::Replace { values: messages.to_vec() },
		     VecDiff::UpdateAt { index: 2, value: new_messages[0].clone() },
		     VecDiff::UpdateAt { index: 3, value: new_messages[1].clone() }],
		vec![VecDiff::Replace { values: notifications.to_vec() }],

		|message, timestamp| {
			if message.timestamp <= timestamp.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	assert_signal_vec_eq(merged, vec![
		Poll::Ready(Some(VecDiff::Replace {
			values: messages.iter().map(to_left).collect(),
		})),

		Poll::Ready(Some(VecDiff::Replace {
			values: vec![
				to_left(&messages[0]),
				to_right(&notifications[0]),
				to_left(&messages[1]),
				to_left(&messages[2]),
				to_left(&messages[3]),
				to_right(&notifications[1]),
				to_right(&notifications[2]),
				to_right(&notifications[3]),
				to_left(&messages[4]),
				to_right(&notifications[4]),
				to_left(&messages[5]),
				to_left(&messages[6]),
				to_left(&messages[7]),
				to_right(&notifications[5]),
			],
		})),

		Poll::Ready(Some(VecDiff::UpdateAt { index: 3, value: to_left(&new_messages[0]) })),
		Poll::Ready(Some(VecDiff::UpdateAt { index: 4, value: to_left(&new_messages[1]) })),
		Poll::Ready(None),
	]);
}

#[test]
fn merge_remove_at() {
	dataset::seed(RNG_SEED);
	let (messages, notifications) = merge_replace_util::initial_dataset();
	let merged = merge_util::signal_from_ops(
		vec![VecDiff::Replace { values: messages.to_vec() },
		     VecDiff::RemoveAt { index: 2 },
		     VecDiff::RemoveAt { index: 3 }],
		vec![VecDiff::Replace { values: notifications.to_vec() }],

		|message, timestamp| {
			if message.timestamp <= timestamp.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	let state = assert_signal_vec_eq(merged, vec![
		Poll::Ready(Some(VecDiff::Replace {
			values: messages.iter().map(to_left).collect(),
		})),

		Poll::Ready(Some(VecDiff::Replace {
			values: vec![
				to_left(&messages[0]),
				to_right(&notifications[0]),
				to_left(&messages[1]),
				to_left(&messages[2]),
				to_left(&messages[3]),
				to_right(&notifications[1]),
				to_right(&notifications[2]),
				to_right(&notifications[3]),
				to_left(&messages[4]),
				to_right(&notifications[4]),
				to_left(&messages[5]),
				to_left(&messages[6]),
				to_left(&messages[7]),
				to_right(&notifications[5]),
			],
		})),

		Poll::Ready(Some(VecDiff::RemoveAt { index: 3 })),
		Poll::Ready(Some(VecDiff::RemoveAt { index: 7 })),
		Poll::Ready(None),
	]);

	assert_eq!(state, vec![
		to_left(&messages[0]),
		to_right(&notifications[0]),
		to_left(&messages[1]),
		to_left(&messages[3]),
		to_right(&notifications[1]),
		to_right(&notifications[2]),
		to_right(&notifications[3]),
		to_right(&notifications[4]),
		to_left(&messages[5]),
		to_left(&messages[6]),
		to_left(&messages[7]),
		to_right(&notifications[5]),
	]);
}

#[test]
fn merge_move_item_left() {
	dataset::seed(RNG_SEED);
	let messages = dataset::messages();
	let notifications = dataset::notifications();

	let merged = merge_util::signal_from_ops(
		vec![VecDiff::Replace { values: messages.to_vec() },
		     VecDiff::Move { old_index: 2, new_index: 7 },
		     VecDiff::Move { old_index: 1, new_index: 4 },
		     VecDiff::Move { old_index: 3, new_index: 5 },
		     VecDiff::Move { old_index: 6, new_index: 1 }],
		vec![VecDiff::Replace { values: notifications.to_vec() }],

		|message, notification| {
			if message.timestamp <= notification.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	let output = assert_signal_vec_eq(merged, vec![
		Poll::Ready(Some(VecDiff::Replace {
			values: messages.iter().map(to_left).collect(),
		})),

		Poll::Ready(Some(VecDiff::Replace {
			values: vec![
				to_left(&messages[0]),
				to_right(&notifications[0]),
				to_left(&messages[1]),
				to_left(&messages[2]),
				to_left(&messages[3]),
				to_right(&notifications[1]),
				to_right(&notifications[2]),
				to_right(&notifications[3]),
				to_left(&messages[4]),
				to_right(&notifications[4]),
				to_left(&messages[5]),
				to_left(&messages[6]),
				to_left(&messages[7]),
				to_right(&notifications[5]),
			],
		})),

		Poll::Ready(Some(VecDiff::Move { old_index: 3, new_index: 12 })),
		Poll::Ready(Some(VecDiff::Move { old_index: 2, new_index: 9 })),
		Poll::Ready(Some(VecDiff::Move { old_index: 8, new_index: 10 })),
		Poll::Ready(Some(VecDiff::Move { old_index: 11, new_index: 2 })),
		Poll::Ready(None),
	]);

	assert_eq!(output, vec![
		to_left(&messages[0]),
		to_right(&notifications[0]),
		to_left(&messages[7]),
		to_left(&messages[3]),
		to_right(&notifications[1]),
		to_right(&notifications[2]),
		to_right(&notifications[3]),
		to_left(&messages[4]),
		to_right(&notifications[4]),
		to_left(&messages[1]),
		to_left(&messages[6]),
		to_left(&messages[5]),
		to_left(&messages[2]),
		to_right(&notifications[5]),
	]);
}

#[test]
fn merge_move_item_right() {
	dataset::seed(RNG_SEED);
	let messages = dataset::messages();
	let notifications = dataset::notifications();

	let merged = merge_util::signal_from_ops(
		vec![VecDiff::Replace { values: messages.to_vec() }],
		vec![VecDiff::Replace { values: notifications.to_vec() },
		     VecDiff::Move { old_index: 0, new_index: 5 },
		     VecDiff::Move { old_index: 2, new_index: 4 },
		     VecDiff::Move { old_index: 1, new_index: 3 },
		     VecDiff::Move { old_index: 0, new_index: 4 },
		     VecDiff::Move { old_index: 3, new_index: 5 },
		     VecDiff::Move { old_index: 1, new_index: 0 }],

		|message, notification| {
			if message.timestamp <= notification.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	let output = assert_signal_vec_eq(merged, vec![
		Poll::Ready(Some(VecDiff::Replace {
			values: messages.iter().map(to_left).collect(),
		})),

		Poll::Ready(Some(VecDiff::Replace {
			values: vec![
				to_left(&messages[0]),
				to_right(&notifications[0]),
				to_left(&messages[1]),
				to_left(&messages[2]),
				to_left(&messages[3]),
				to_right(&notifications[1]),
				to_right(&notifications[2]),
				to_right(&notifications[3]),
				to_left(&messages[4]),
				to_right(&notifications[4]),
				to_left(&messages[5]),
				to_left(&messages[6]),
				to_left(&messages[7]),
				to_right(&notifications[5]),
			],
		})),

		Poll::Ready(Some(VecDiff::Move { old_index: 1, new_index: 13 })),
		Poll::Ready(Some(VecDiff::Move { old_index: 6, new_index: 12 })),
		Poll::Ready(Some(VecDiff::Move { old_index: 5, new_index: 11 })),
		Poll::Ready(Some(VecDiff::Move { old_index: 4, new_index: 12 })),
		Poll::Ready(Some(VecDiff::Move { old_index: 11, new_index: 13 })),
		Poll::Ready(Some(VecDiff::Move { old_index: 9, new_index: 5 })),
		Poll::Ready(None),
	]);

	assert_eq!(output, vec![
		to_left(&messages[0]),
		to_left(&messages[1]),
		to_left(&messages[2]),
		to_left(&messages[3]),
		to_left(&messages[4]),
		to_right(&notifications[5]),
		to_right(&notifications[4]),
		to_left(&messages[5]),
		to_left(&messages[6]),
		to_left(&messages[7]),
		to_right(&notifications[2]),
		to_right(&notifications[1]),
		to_right(&notifications[0]),
		to_right(&notifications[3]),
	]);
}

#[test]
fn merge_move_item_interleaved() {
	#[derive(Debug, Clone)]
	struct Move {
		old_index: usize,
		new_index: usize,
	}

	dataset::seed(RNG_SEED);
	let messages = dataset::messages();
	let notifications = dataset::notifications();

	let messages_vec = MutableVec::new_with_values(messages.to_vec());
	let notifications_vec = MutableVec::new_with_values(notifications.to_vec());

	let mut merged = merge_util::signal_from_sources(
		messages_vec.signal_vec_cloned(),
		notifications_vec.signal_vec_cloned(),

		|message, notification| {
			if message.timestamp <= notification.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	let moves = vec![
		into_left(  Move { old_index: 2, new_index: 7 } ),
		into_right( Move { old_index: 0, new_index: 5 } ),
		into_right( Move { old_index: 2, new_index: 4 } ),
		into_left(  Move { old_index: 1, new_index: 4 } ),
		into_right( Move { old_index: 1, new_index: 3 } ),
		into_left(  Move { old_index: 3, new_index: 5 } ),
		into_right( Move { old_index: 0, new_index: 4 } ),
		into_right( Move { old_index: 3, new_index: 5 } ),
		into_left(  Move { old_index: 6, new_index: 1 } ),
		into_right( Move { old_index: 1, new_index: 0 } ),
	];

	let expected_moves = vec![
		VecDiff::Move { old_index:  3, new_index: 12 },
		VecDiff::Move { old_index:  1, new_index: 13 },
		VecDiff::Move { old_index:  5, new_index: 12 },
		VecDiff::Move { old_index:  1, new_index:  7 },
		VecDiff::Move { old_index:  3, new_index: 11 },
		VecDiff::Move { old_index:  5, new_index:  7 },
		VecDiff::Move { old_index:  2, new_index: 12 },
		VecDiff::Move { old_index: 11, new_index: 13 },
		VecDiff::Move { old_index:  7, new_index:  1 },
		VecDiff::Move { old_index:  9, new_index:  4 },
	];

	let mut output = Vec::new();
	with_noop_context(|cx| {
		let mut messages_lock = messages_vec.lock_mut();
		let mut notifications_lock = notifications_vec.lock_mut();

		#[track_caller]
		fn test_apply(
			actual: Poll<Option<VecDiff<MergedVecItem<Rc<dataset::Message>, Rc<dataset::Notification>>>>>,
			expected: Poll<Option<VecDiff<MergedVecItem<Rc<dataset::Message>, Rc<dataset::Notification>>>>>,
			vec: &mut Vec<MergedVecItem<Rc<dataset::Message>, Rc<dataset::Notification>>>,
		) {
			let location = std::panic::Location::caller();

			log::trace!(
				"{file}:{line} [{module}::merge_move_item_interleaved]\n  expected was {expected:?}\n  actual was {actual:?}\n",
				file = location.file(), line = location.line(), module = module_path!(),
			);

			assert_eq!(expected, actual);
			let item = match actual {
				Poll::Ready(item) => item,
				_ => unimplemented!("item should be ready"),
			};

			item.expect("a non-empty change").apply_to_vec(vec);
		}

		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Replace {
				values: messages.iter().map(to_left).collect(),
			})),
			&mut output,
		);

		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Replace {
				values: vec![
					to_left(&messages[0]),
					to_right(&notifications[0]),
					to_left(&messages[1]),
					to_left(&messages[2]),
					to_left(&messages[3]),
					to_right(&notifications[1]),
					to_right(&notifications[2]),
					to_right(&notifications[3]),
					to_left(&messages[4]),
					to_right(&notifications[4]),
					to_left(&messages[5]),
					to_left(&messages[6]),
					to_left(&messages[7]),
					to_right(&notifications[5]),
				],
			})),
			&mut output,
		);

		moves.into_iter()
			.zip(expected_moves.into_iter())
			.for_each(|(op, expected_transform)| {
				match op {
					MergedVecItem::Left(Move { old_index, new_index }) => messages_lock.move_from_to(old_index, new_index),
					MergedVecItem::Right(Move { old_index, new_index }) => notifications_lock.move_from_to(old_index, new_index),
				};

				test_apply(
					merged.poll_vec_change_unpin(cx),
					Poll::Ready(Some(expected_transform)),
					&mut output,
				);
			});

		// we're done with `messages`, drop the MutableVec
		drop(messages_lock);
		drop(messages_vec);

		// we're done with `notifications`, drop the MutableVec
		drop(notifications_lock);
		drop(notifications_vec);

		// ensure Ready(None) is emitted correctly
		assert_eq!(Poll::Ready(None), merged.poll_vec_change_unpin(cx));
	});

	assert_eq!(output, vec![
		to_left(&messages[0]),
		to_left(&messages[7]),
		to_left(&messages[3]),
		to_left(&messages[4]),
		to_right(&notifications[5]),
		to_right(&notifications[4]),
		to_left(&messages[1]),
		to_left(&messages[6]),
		to_left(&messages[5]),
		to_left(&messages[2]),
		to_right(&notifications[2]),
		to_right(&notifications[1]),
		to_right(&notifications[0]),
		to_right(&notifications[3]),
	]);
}

#[test]
fn merge_push_in_order() {
	dataset::seed(RNG_SEED);
	let messages = dataset::messages();
	let notifications = dataset::notifications();

	let messages_vec: MutableVec<Rc<dataset::Message>> = MutableVec::new();
	let notifications_vec: MutableVec<Rc<dataset::Notification>> = MutableVec::new();

	let mut merged = merge_util::signal_from_sources(
		messages_vec.signal_vec_cloned(),
		notifications_vec.signal_vec_cloned(),

		|message, timestamp| {
			if message.timestamp <= timestamp.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	let mut output = Vec::new();
	with_noop_context(|cx| {
		let mut messages_lock = messages_vec.lock_mut();
		let mut notifications_lock = notifications_vec.lock_mut();

		#[track_caller]
		fn test_apply(
			actual: Poll<Option<VecDiff<MergedVecItem<Rc<dataset::Message>, Rc<dataset::Notification>>>>>,
			expected: Poll<Option<VecDiff<MergedVecItem<Rc<dataset::Message>, Rc<dataset::Notification>>>>>,
			vec: &mut Vec<MergedVecItem<Rc<dataset::Message>, Rc<dataset::Notification>>>,
		) {
			let location = std::panic::Location::caller();

			log::trace!(
				"{file}:{line} [{module}::merge_push_in_order]\n  expected was {expected:?}\n  actual was {actual:?}\n",
				file = location.file(), line = location.line(), module = module_path!(),
			);

			assert_eq!(expected, actual);
			let item = match actual {
				Poll::Ready(item) => item,
				_ => unimplemented!("item should be ready"),
			};

			item.expect("a non-empty change").apply_to_vec(vec);
		}

		messages_lock.push_cloned(messages[0].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_left(&messages[0]) })),
			&mut output,
		);

		notifications_lock.push_cloned(notifications[0].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_right(&notifications[0]) })),
			&mut output,
		);

		messages_lock.push_cloned(messages[1].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_left(&messages[1]) })),
			&mut output,
		);

		messages_lock.push_cloned(messages[2].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_left(&messages[2]) })),
			&mut output,
		);

		messages_lock.push_cloned(messages[3].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_left(&messages[3]) })),
			&mut output,
		);

		notifications_lock.push_cloned(notifications[1].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_right(&notifications[1]) })),
			&mut output,
		);

		notifications_lock.push_cloned(notifications[2].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_right(&notifications[2]) })),
			&mut output,
		);

		notifications_lock.push_cloned(notifications[3].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_right(&notifications[3]) })),
			&mut output,
		);

		messages_lock.push_cloned(messages[4].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_left(&messages[4]) })),
			&mut output,
		);

		notifications_lock.push_cloned(notifications[4].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_right(&notifications[4]) })),
			&mut output,
		);

		messages_lock.push_cloned(messages[5].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_left(&messages[5]) })),
			&mut output,
		);

		messages_lock.push_cloned(messages[6].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_left(&messages[6]) })),
			&mut output,
		);

		messages_lock.push_cloned(messages[7].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_left(&messages[7]) })),
			&mut output,
		);

		// we're done with `messages`, drop the MutableVec
		drop(messages_lock);
		drop(messages_vec);

		notifications_lock.push_cloned(notifications[5].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_right(&notifications[5]) })),
			&mut output,
		);

		// we're done with `notifications`, drop the MutableVec
		drop(notifications_lock);
		drop(notifications_vec);

		// ensure Ready(None) is emitted correctly
		assert_eq!(Poll::Ready(None), merged.poll_vec_change_unpin(cx));
	});

	// ensure that the final list is as expected
	assert_eq!(output, vec![
		to_left(&messages[0]),
		to_right(&notifications[0]),
		to_left(&messages[1]),
		to_left(&messages[2]),
		to_left(&messages[3]),
		to_right(&notifications[1]),
		to_right(&notifications[2]),
		to_right(&notifications[3]),
		to_left(&messages[4]),
		to_right(&notifications[4]),
		to_left(&messages[5]),
		to_left(&messages[6]),
		to_left(&messages[7]),
		to_right(&notifications[5]),
	]);
}

#[test]
fn merge_push_out_of_order() {
	dataset::seed(RNG_SEED);
	let messages = dataset::messages();
	let notifications = dataset::notifications();

	let messages_vec: MutableVec<Rc<dataset::Message>> = MutableVec::new();
	let notifications_vec: MutableVec<Rc<dataset::Notification>> = MutableVec::new();

	let mut merged = merge_util::signal_from_sources(
		messages_vec.signal_vec_cloned(),
		notifications_vec.signal_vec_cloned(),

		|message, timestamp| {
			if message.timestamp <= timestamp.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	let mut output = Vec::new();
	with_noop_context(|cx| {
		let mut messages_lock = messages_vec.lock_mut();
		let mut notifications_lock = notifications_vec.lock_mut();

		#[track_caller]
		fn test_apply(
			actual: Poll<Option<VecDiff<MergedVecItem<Rc<dataset::Message>, Rc<dataset::Notification>>>>>,
			expected: Poll<Option<VecDiff<MergedVecItem<Rc<dataset::Message>, Rc<dataset::Notification>>>>>,
			vec: &mut Vec<MergedVecItem<Rc<dataset::Message>, Rc<dataset::Notification>>>,
		) {
			let location = std::panic::Location::caller();

			log::trace!(
				"{file}:{line} [{module}::merge_push_out_of_order]\n  expected was {expected:?}\n  actual was {actual:?}\n",
				file = location.file(), line = location.line(), module = module_path!(),
			);

			assert_eq!(expected, actual);
			let item = match actual {
				Poll::Ready(item) => item,
				_ => unimplemented!("item should be ready"),
			};

			item.expect("a non-empty change").apply_to_vec(vec);
		}

		messages_lock.push_cloned(messages[0].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_left(&messages[0]) })),
			&mut output,
		);

		notifications_lock.push_cloned(notifications[0].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_right(&notifications[0]) })),
			&mut output,
		);

		messages_lock.push_cloned(messages[1].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_left(&messages[1]) })),
			&mut output,
		);

		messages_lock.push_cloned(messages[2].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_left(&messages[2]) })),
			&mut output,
		);

		notifications_lock.push_cloned(notifications[1].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_right(&notifications[1]) })),
			&mut output,
		);

		messages_lock.push_cloned(messages[3].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::InsertAt { index: 4, value: to_left(&messages[3]) })),
			&mut output,
		);

		notifications_lock.push_cloned(notifications[2].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_right(&notifications[2]) })),
			&mut output,
		);

		notifications_lock.push_cloned(notifications[3].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_right(&notifications[3]) })),
			&mut output,
		);

		notifications_lock.push_cloned(notifications[4].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_right(&notifications[4]) })),
			&mut output,
		);

		messages_lock.push_cloned(messages[4].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::InsertAt { index: 8, value: to_left(&messages[4]) })),
			&mut output,
		);

		messages_lock.push_cloned(messages[5].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_left(&messages[5]) })),
			&mut output,
		);

		messages_lock.push_cloned(messages[6].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_left(&messages[6]) })),
			&mut output,
		);

		messages_lock.push_cloned(messages[7].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_left(&messages[7]) })),
			&mut output,
		);

		// we're done with `messages`, drop the MutableVec
		drop(messages_lock);
		drop(messages_vec);

		notifications_lock.push_cloned(notifications[5].clone());
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Push { value: to_right(&notifications[5]) })),
			&mut output,
		);

		// we're done with `notifications`, drop the MutableVec
		drop(notifications_lock);
		drop(notifications_vec);

		// ensure Ready(None) is emitted correctly
		assert_eq!(Poll::Ready(None), merged.poll_vec_change_unpin(cx));
	});

	// ensure that the final list is as expected
	assert_eq!(output, vec![
		to_left(&messages[0]),
		to_right(&notifications[0]),
		to_left(&messages[1]),
		to_left(&messages[2]),
		to_left(&messages[3]),
		to_right(&notifications[1]),
		to_right(&notifications[2]),
		to_right(&notifications[3]),
		to_left(&messages[4]),
		to_right(&notifications[4]),
		to_left(&messages[5]),
		to_left(&messages[6]),
		to_left(&messages[7]),
		to_right(&notifications[5]),
	]);
}

#[test]
fn merge_pop_left() {
	dataset::seed(RNG_SEED);
	let messages = dataset::messages();
	let notifications = dataset::notifications();

	let merged = merge_util::signal_from_ops(
		vec![VecDiff::Replace { values: messages.to_vec() },
		     VecDiff::Pop {},
		     VecDiff::Pop {},
		     VecDiff::Pop {},
		     VecDiff::Pop {},
		     VecDiff::Pop {},
		     VecDiff::Pop {},
		     VecDiff::Pop {},
		     VecDiff::Pop {}],
		vec![VecDiff::Replace { values: notifications.to_vec() }],

		|message, notification| {
			if message.timestamp <= notification.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	let output = assert_signal_vec_eq(merged, vec![
		Poll::Ready(Some(VecDiff::Replace {
			values: messages.iter().map(to_left).collect(),
		})),

		Poll::Ready(Some(VecDiff::Replace {
			values: vec![
				to_left(&messages[0]),
				to_right(&notifications[0]),
				to_left(&messages[1]),
				to_left(&messages[2]),
				to_left(&messages[3]),
				to_right(&notifications[1]),
				to_right(&notifications[2]),
				to_right(&notifications[3]),
				to_left(&messages[4]),
				to_right(&notifications[4]),
				to_left(&messages[5]),
				to_left(&messages[6]),
				to_left(&messages[7]),
				to_right(&notifications[5]),
			],
		})),

		Poll::Ready(Some(VecDiff::RemoveAt { index: 12 })),
		Poll::Ready(Some(VecDiff::RemoveAt { index: 11 })),
		Poll::Ready(Some(VecDiff::RemoveAt { index: 10 })),
		Poll::Ready(Some(VecDiff::RemoveAt { index:  8 })),
		Poll::Ready(Some(VecDiff::RemoveAt { index:  4 })),
		Poll::Ready(Some(VecDiff::RemoveAt { index:  3 })),
		Poll::Ready(Some(VecDiff::RemoveAt { index:  2 })),
		Poll::Ready(Some(VecDiff::RemoveAt { index:  0 })),
		Poll::Ready(None),
	]);

	let notifications_only: Vec<_> = notifications.iter().map(to_right).collect();
	assert_eq!(output, notifications_only);
}

#[test]
fn merge_pop_right() {
	dataset::seed(RNG_SEED);
	let messages = dataset::messages();
	let notifications = dataset::notifications();

	let merged = merge_util::signal_from_ops(
		vec![VecDiff::Replace { values: messages.to_vec() }],
		vec![VecDiff::Replace { values: notifications.to_vec() },
		     VecDiff::Pop {},
		     VecDiff::Pop {},
		     VecDiff::Pop {},
		     VecDiff::Pop {},
		     VecDiff::Pop {},
		     VecDiff::Pop {}],

		|message, notification| {
			if message.timestamp <= notification.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	let output = assert_signal_vec_eq(merged, vec![
		Poll::Ready(Some(VecDiff::Replace {
			values: messages.iter().map(to_left).collect(),
		})),

		Poll::Ready(Some(VecDiff::Replace {
			values: vec![
				to_left(&messages[0]),
				to_right(&notifications[0]),
				to_left(&messages[1]),
				to_left(&messages[2]),
				to_left(&messages[3]),
				to_right(&notifications[1]),
				to_right(&notifications[2]),
				to_right(&notifications[3]),
				to_left(&messages[4]),
				to_right(&notifications[4]),
				to_left(&messages[5]),
				to_left(&messages[6]),
				to_left(&messages[7]),
				to_right(&notifications[5]),
			],
		})),

		Poll::Ready(Some(VecDiff::Pop {})),
		Poll::Ready(Some(VecDiff::RemoveAt { index: 9 })),
		Poll::Ready(Some(VecDiff::RemoveAt { index: 7 })),
		Poll::Ready(Some(VecDiff::RemoveAt { index: 6 })),
		Poll::Ready(Some(VecDiff::RemoveAt { index: 5 })),
		Poll::Ready(Some(VecDiff::RemoveAt { index: 1 })),
		Poll::Ready(None),
	]);

	let messages_only: Vec<_> = messages.iter().map(to_left).collect();
	assert_eq!(output, messages_only);
}

#[test]
fn merge_pop_all_interleaved() {
	dataset::seed(RNG_SEED);
	let messages = dataset::messages();
	let notifications = dataset::notifications();

	let messages_vec = MutableVec::new_with_values(messages.to_vec());
	let notifications_vec = MutableVec::new_with_values(notifications.to_vec());

	let mut merged = merge_util::signal_from_sources(
		messages_vec.signal_vec_cloned(),
		notifications_vec.signal_vec_cloned(),

		|message, timestamp| {
			if message.timestamp <= timestamp.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	let mut output = Vec::new();
	with_noop_context(|cx| {
		let mut messages_lock = messages_vec.lock_mut();
		let mut notifications_lock = notifications_vec.lock_mut();

		#[track_caller]
		fn test_apply(
			actual: Poll<Option<VecDiff<MergedVecItem<Rc<dataset::Message>, Rc<dataset::Notification>>>>>,
			expected: Poll<Option<VecDiff<MergedVecItem<Rc<dataset::Message>, Rc<dataset::Notification>>>>>,
			vec: &mut Vec<MergedVecItem<Rc<dataset::Message>, Rc<dataset::Notification>>>,
		) {
			let location = std::panic::Location::caller();

			log::trace!(
				"{file}:{line} [{module}::merge_pop_all_interleaved]\n  expected was {expected:?}\n  actual was {actual:?}\n",
				file = location.file(), line = location.line(), module = module_path!(),
			);

			assert_eq!(expected, actual);
			let item = match actual {
				Poll::Ready(item) => item,
				_ => unimplemented!("item should be ready"),
			};

			item.expect("a non-empty change").apply_to_vec(vec);
		}

		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Replace {
				values: messages.iter().map(to_left).collect(),
			})),
			&mut output,
		);

		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Replace {
				values: vec![
					to_left(&messages[0]),
					to_right(&notifications[0]),
					to_left(&messages[1]),
					to_left(&messages[2]),
					to_left(&messages[3]),
					to_right(&notifications[1]),
					to_right(&notifications[2]),
					to_right(&notifications[3]),
					to_left(&messages[4]),
					to_right(&notifications[4]),
					to_left(&messages[5]),
					to_left(&messages[6]),
					to_left(&messages[7]),
					to_right(&notifications[5]),
				],
			})),
			&mut output,
		);

		let _ = messages_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::RemoveAt { index: 12 })),
			&mut output,
		);

		let _ = notifications_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = messages_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = notifications_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::RemoveAt { index: 9 })),
			&mut output,
		);

		let _ = messages_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = notifications_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::RemoveAt { index: 7 })),
			&mut output,
		);

		let _ = messages_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = notifications_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = messages_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::RemoveAt { index: 4 })),
			&mut output,
		);

		let _ = notifications_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = messages_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = notifications_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::RemoveAt { index: 1 })),
			&mut output,
		);

		// we're done with `notifications`, drop the MutableVec
		drop(notifications_lock);
		drop(notifications_vec);

		let _ = messages_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = messages_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		// we're done with `messages`, drop the MutableVec
		drop(messages_lock);
		drop(messages_vec);

		// ensure Ready(None) is emitted correctly
		assert_eq!(Poll::Ready(None), merged.poll_vec_change_unpin(cx));
	});

	// ensure that the final list is empty
	assert_eq!(output, vec![]);
}

#[test]
fn merge_pop_all_ordered() {
	dataset::seed(RNG_SEED);
	let messages = dataset::messages();
	let notifications = dataset::notifications();

	let messages_vec = MutableVec::new_with_values(messages.to_vec());
	let notifications_vec = MutableVec::new_with_values(notifications.to_vec());

	let mut merged = merge_util::signal_from_sources(
		messages_vec.signal_vec_cloned(),
		notifications_vec.signal_vec_cloned(),

		|message, timestamp| {
			if message.timestamp <= timestamp.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	let mut output = Vec::new();
	with_noop_context(|cx| {
		let mut messages_lock = messages_vec.lock_mut();
		let mut notifications_lock = notifications_vec.lock_mut();

		#[track_caller]
		fn test_apply(
			actual: Poll<Option<VecDiff<MergedVecItem<Rc<dataset::Message>, Rc<dataset::Notification>>>>>,
			expected: Poll<Option<VecDiff<MergedVecItem<Rc<dataset::Message>, Rc<dataset::Notification>>>>>,
			vec: &mut Vec<MergedVecItem<Rc<dataset::Message>, Rc<dataset::Notification>>>,
		) {
			let location = std::panic::Location::caller();

			log::trace!(
				"{file}:{line} [{module}::merge_pop_all_ordered]\n  expected was {expected:?}\n  actual was {actual:?}\n",
				file = location.file(), line = location.line(), module = module_path!(),
			);

			assert_eq!(expected, actual);
			let item = match actual {
				Poll::Ready(item) => item,
				_ => unimplemented!("item should be ready"),
			};

			item.expect("a non-empty change").apply_to_vec(vec);
		}

		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Replace {
				values: messages.iter().map(to_left).collect(),
			})),
			&mut output,
		);

		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Replace {
				values: vec![
					to_left(&messages[0]),
					to_right(&notifications[0]),
					to_left(&messages[1]),
					to_left(&messages[2]),
					to_left(&messages[3]),
					to_right(&notifications[1]),
					to_right(&notifications[2]),
					to_right(&notifications[3]),
					to_left(&messages[4]),
					to_right(&notifications[4]),
					to_left(&messages[5]),
					to_left(&messages[6]),
					to_left(&messages[7]),
					to_right(&notifications[5]),
				],
			})),
			&mut output,
		);

		let _ = notifications_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = messages_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = messages_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = messages_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = notifications_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = messages_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = notifications_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = notifications_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = notifications_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = messages_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = messages_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = messages_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		let _ = notifications_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		// we're done with `notifications`, drop the MutableVec
		drop(notifications_lock);
		drop(notifications_vec);

		let _ = messages_lock.pop();
		test_apply(
			merged.poll_vec_change_unpin(cx),
			Poll::Ready(Some(VecDiff::Pop {})),
			&mut output,
		);

		// we're done with `messages`, drop the MutableVec
		drop(messages_lock);
		drop(messages_vec);

		// ensure Ready(None) is emitted correctly
		assert_eq!(Poll::Ready(None), merged.poll_vec_change_unpin(cx));
	});

	// ensure that the final list is empty
	assert_eq!(output, vec![]);
}

#[test]
fn merge_clear_left_first() {
	dataset::seed(RNG_SEED);
	let messages = dataset::messages();
	let notifications = dataset::notifications();

	let messages_vec = MutableVec::new_with_values(messages.to_vec());
	let notifications_vec = MutableVec::new_with_values(notifications.to_vec());

	let mut merged = merge_util::signal_from_sources(
		messages_vec.signal_vec_cloned(),
		notifications_vec.signal_vec_cloned(),

		|message, notification| {
			if message.timestamp <= notification.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	with_noop_context(|cx| {
		let mut messages_lock = messages_vec.lock_mut();
		let mut notifications_lock = notifications_vec.lock_mut();

		assert_eq!(
			Poll::Ready(Some(VecDiff::Replace {
				values: messages.iter().map(to_left).collect(),
			})),

			merged.poll_vec_change_unpin(cx),
		);

		assert_eq!(
			Poll::Ready(Some(VecDiff::Replace {
				values: vec![
					to_left(&messages[0]),
					to_right(&notifications[0]),
					to_left(&messages[1]),
					to_left(&messages[2]),
					to_left(&messages[3]),
					to_right(&notifications[1]),
					to_right(&notifications[2]),
					to_right(&notifications[3]),
					to_left(&messages[4]),
					to_right(&notifications[4]),
					to_left(&messages[5]),
					to_left(&messages[6]),
					to_left(&messages[7]),
					to_right(&notifications[5]),
				],
			})),

			merged.poll_vec_change_unpin(cx),
		);

		messages_lock.clear();
		assert_eq!(
			Poll::Ready(Some(VecDiff::Replace {
				values: notifications.iter().map(to_right).collect(),
			})),

			merged.poll_vec_change_unpin(cx),
		);

		// we're done with `messages`, drop the MutableVec
		drop(messages_lock);
		drop(messages_vec);

		notifications_lock.clear();
		assert_eq!(Poll::Ready(Some(VecDiff::Clear {})), merged.poll_vec_change_unpin(cx));

		// we're done with `notifications`, drop the MutableVec
		drop(notifications_lock);
		drop(notifications_vec);

		// ensure Ready(None) is emitted correctly
		assert_eq!(Poll::Ready(None), merged.poll_vec_change_unpin(cx));
	});
}

#[test]
fn merge_clear_right_first() {
	dataset::seed(RNG_SEED);
	let messages = dataset::messages();
	let notifications = dataset::notifications();

	let messages_vec = MutableVec::new_with_values(messages.to_vec());
	let notifications_vec = MutableVec::new_with_values(notifications.to_vec());

	let mut merged = merge_util::signal_from_sources(
		messages_vec.signal_vec_cloned(),
		notifications_vec.signal_vec_cloned(),

		|message, notification| {
			if message.timestamp <= notification.timestamp {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	);

	with_noop_context(|cx| {
		let mut messages_lock = messages_vec.lock_mut();
		let mut notifications_lock = notifications_vec.lock_mut();

		assert_eq!(
			Poll::Ready(Some(VecDiff::Replace {
				values: messages.iter().map(to_left).collect(),
			})),

			merged.poll_vec_change_unpin(cx),
		);

		assert_eq!(
			Poll::Ready(Some(VecDiff::Replace {
				values: vec![
					to_left(&messages[0]),
					to_right(&notifications[0]),
					to_left(&messages[1]),
					to_left(&messages[2]),
					to_left(&messages[3]),
					to_right(&notifications[1]),
					to_right(&notifications[2]),
					to_right(&notifications[3]),
					to_left(&messages[4]),
					to_right(&notifications[4]),
					to_left(&messages[5]),
					to_left(&messages[6]),
					to_left(&messages[7]),
					to_right(&notifications[5]),
				],
			})),

			merged.poll_vec_change_unpin(cx),
		);

		notifications_lock.clear();
		assert_eq!(
			Poll::Ready(Some(VecDiff::Replace {
				values: messages.iter().map(to_left).collect(),
			})),

			merged.poll_vec_change_unpin(cx),
		);

		// we're done with `notifications`, drop the MutableVec
		drop(notifications_lock);
		drop(notifications_vec);

		messages_lock.clear();
		assert_eq!(Poll::Ready(Some(VecDiff::Clear {})), merged.poll_vec_change_unpin(cx));

		// we're done with `messages`, drop the MutableVec
		drop(messages_lock);
		drop(messages_vec);

		// ensure Ready(None) is emitted correctly
		assert_eq!(Poll::Ready(None), merged.poll_vec_change_unpin(cx));
	});
}

fn into_left<Left, Right>(item: Left) -> MergedVecItem<Left, Right>
where Left: Debug,
      Right: Debug,
{
	MergedVecItem::Left(item)
}

fn into_right<Left, Right>(item: Right) -> MergedVecItem<Left, Right>
where Left: Debug,
      Right: Debug,
{
	MergedVecItem::Right(item)
}

fn to_left<Left, Right>(item: &Left) -> MergedVecItem<Left, Right>
where Left: Debug + Clone,
      Right: Debug + Clone,
{
	into_left(item.clone())
}

fn to_right<Left, Right>(item: &Right) -> MergedVecItem<Left, Right>
where Left: Debug + Clone,
      Right: Debug + Clone,
{
	into_right(item.clone())
}

mod dataset {
	use chrono::{DateTime, NaiveDateTime, ParseResult, TimeZone, Utc};
	use fake::Fake;
	use fake::faker::lorem::en::Sentences;
	use fake::faker::name::en::Name;
	use rand::SeedableRng;
	use rand::rngs::StdRng;
	use once_cell::unsync::OnceCell;
	use std::cell::RefCell;
	use std::fmt;
	use std::rc::Rc;

	thread_local! {
		static RNG: RefCell<OnceCell<StdRng>> = RefCell::new(OnceCell::new());
	}

	#[derive(Eq, PartialEq)]
	pub struct Message {
		pub username: String,
		pub message: String,
		pub timestamp: DateTime<Utc>,
	}

	impl Message {
		pub fn new(timestamp: DateTime<Utc>) -> Rc<Self> {
			RNG.with(|cell| {
				let mut cell = cell.borrow_mut();
				let rng = cell.get_mut().expect("rng should be seeded with data::seed()");

				Rc::new(Message {
					timestamp,

					username: Name().fake_with_rng(rng),
					message: Sentences(1..6).fake_with_rng::<Vec<String>, _>(rng).join(" "),
				})
			})
		}

		pub fn reroll(existing: &Message) -> Rc<Self> {
			RNG.with(|cell| {
				let mut cell = cell.borrow_mut();
				let rng = cell.get_mut().expect("rng should be seeded with data::seed()");

				Rc::new(Message {
					username: Name().fake_with_rng(rng),
					message: Sentences(1..6).fake_with_rng::<Vec<String>, _>(rng).join(" "),
					timestamp: existing.timestamp.clone(),
				})
			})
		}
	}

	impl fmt::Debug for Message {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_tuple("Message")
				.field(&self.timestamp)
				.finish()
		}
	}

	impl fmt::Display for Message {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_tuple("Message")
				.field(&self.timestamp)
				.finish()
		}
	}

	#[derive(Eq, PartialEq)]
	pub struct Notification {
		pub message: String,
		pub timestamp: DateTime<Utc>,
	}

	impl Notification {
		pub fn new(timestamp: DateTime<Utc>) -> Rc<Self> {
			RNG.with(|cell| {
				let mut cell = cell.borrow_mut();
				let rng = cell.get_mut().expect("rng should be seeded with data::seed()");

				Rc::new(Notification {
					timestamp,

					message: Sentences(1..2).fake_with_rng::<Vec<String>, _>(rng).join(" ")
				})
			})
		}
	}

	impl fmt::Debug for Notification {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_tuple("Notification")
				.field(&self.timestamp)
				.finish()
		}
	}

	impl fmt::Display for Notification {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_tuple("Notification")
				.field(&self.timestamp)
				.finish()
		}
	}

	pub fn messages() -> [Rc<Message>; 8] {
		let messages = [
			Message::new(datetime_from_str("2022-07-17 03:31:13").expect("datestr should be valid")),
			Message::new(datetime_from_str("2022-07-17 06:13:38").expect("datestr should be valid")),
			Message::new(datetime_from_str("2022-07-17 12:02:11").expect("datestr should be valid")),
			Message::new(datetime_from_str("2022-07-17 12:13:40").expect("datestr should be valid")),
			Message::new(datetime_from_str("2022-07-17 17:50:08").expect("datestr should be valid")),
			Message::new(datetime_from_str("2022-07-17 18:45:29").expect("datestr should be valid")),
			Message::new(datetime_from_str("2022-07-17 19:03:06").expect("datestr should be valid")),
			Message::new(datetime_from_str("2022-07-17 19:36:29").expect("datestr should be valid")),
		];

		messages
	}

	pub fn notifications() -> [Rc<Notification>; 6] {
		let notifications = [
			Notification::new(datetime_from_str("2022-07-17 05:40:51").expect("datestr should be valid")),
			Notification::new(datetime_from_str("2022-07-17 14:50:43").expect("datestr should be valid")),
			Notification::new(datetime_from_str("2022-07-17 15:31:53").expect("datestr should be valid")),
			Notification::new(datetime_from_str("2022-07-17 16:59:48").expect("datestr should be valid")),
			Notification::new(datetime_from_str("2022-07-17 18:08:59").expect("datestr should be valid")),
			Notification::new(datetime_from_str("2022-07-17 21:21:57").expect("datestr should be valid")),
		];

		notifications
	}

	pub fn datetime_from_str(datestr: &str) -> ParseResult<DateTime<Utc>> {
		NaiveDateTime::parse_from_str(datestr, "%Y-%m-%d %H:%M:%S").map(|naive| Utc.from_local_datetime(&naive).unwrap())
	}

	pub fn seed(seed: u64) {
		RNG.with(|cell| cell.borrow_mut().set(StdRng::seed_from_u64(seed))).expect("rng should not be seeded at this point");
	}
}

mod merge_replace_util {
    use crate::dataset::{self, Message, Notification};
    use std::rc::Rc;

	pub fn initial_dataset() -> ([Rc<Message>; 8], [Rc<Notification>; 6]) {
		let messages = dataset::messages();
		let notifications = dataset::notifications();

		(messages, notifications)
	}
}

mod merge_util {
	use crate::util::Source;
	use futures_signals::signal_vec::{SignalVec, VecDiff};
	use snowcat_signals::signal_vec::SnowcatSignalVecExt;
	use snowcat_signals::signal_vec::merge::Merge2;
	use std::cmp::Ordering;
	use std::fmt::Debug;
	use std::task::Poll;

	pub fn signal_from_initial<Left, Right, OrderFn>(
		left_initial: Vec<Left>,
		right_initial: Vec<Right>,
		order_fn: OrderFn,
	) -> Merge2<Source<VecDiff<Left>>, Source<VecDiff<Right>>, OrderFn>
	where Left: Debug + Clone + PartialEq,
	      Right: Debug + Clone + PartialEq,
	      OrderFn: Fn(&Left, &Right) -> Ordering
	{
		signal_from_ops(
			vec![VecDiff::Replace { values: left_initial }],
			vec![VecDiff::Replace { values: right_initial }],
			order_fn,
		)
	}

	pub fn signal_from_ops<Left, Right, OrderFn>(
		left: Vec<VecDiff<Left>>,
		right: Vec<VecDiff<Right>>,
		order_fn: OrderFn,
	) -> Merge2<Source<VecDiff<Left>>, Source<VecDiff<Right>>, OrderFn>
	where Left: Debug + Clone + PartialEq,
	      Right: Debug + Clone + PartialEq,
	      OrderFn: Fn(&Left, &Right) -> Ordering,
	{
		let left_source: Source<VecDiff<Left>> = Source::new(
			left.into_iter().map(|value| Poll::Ready(value)).collect()
		);

		let right_source: Source<VecDiff<Right>> = Source::new(
			right.into_iter().map(|value| Poll::Ready(value)).collect()
		);

		let merged = left_source.merge(right_source, order_fn);

		merged
	}

	pub fn signal_from_sources<LeftSource, RightSource, Left, Right, OrderFn>(
		left_source: LeftSource,
		right_source: RightSource,
		order_fn: OrderFn,
	) -> Merge2<LeftSource, RightSource, OrderFn>
	where LeftSource: SignalVec<Item = Left>,
	      RightSource: SignalVec<Item = Right>,
		  Left: Debug + Clone + PartialEq,
		  Right: Debug + Clone + PartialEq,
	      OrderFn: Fn(&Left, &Right) -> Ordering,
	{
		left_source.merge(right_source, order_fn)
	}
}
