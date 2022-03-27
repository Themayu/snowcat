/// A pre-existing set of owned characters for testing purposes.
pub(super) const CHARACTERS: [(u64, &'static str); 5] = [
	(2543, "Marabel Thorne"),
	(191498, "Markelio"),
	(273593, "Phoney Baloney"),
	(327067, "Sarah Blitz Garrison"),
	(68851, "Yanozo Serna"),
];

/// The default character to use when testing.
pub(super) const DEFAULT_CHARACTER: u64 = 327067;

/// A pre-existing set of friend bindings for testing purposes.
pub(super) const FRIENDS: [(u64, &'static [&'static str]); 5] = [
	(2543, &["Andrew Kane", "Anthony", "Corny Corn", "Lilia Norse", "Parrot Clara"]), // Marabel Thorne: 5
	(191498, &["Parrot Clara"]), // Markelio: 1
	(273593, &["Andrew Kane", "Corny Corn", "Lilia Norse", "Parrot Clara"]), // Phoney Baloney: 4
	(327067, &["Andrew Kane", "Anthony", "Corny Corn", "Parrot Clara"]), // Sara Blitz Garrison: 4
	(68851, &["Andrew Kane", "Corny Corn", "Lilia Norse"]), // Yanozo Serna: 3
];
