enum Display {
	ShowHand(u8),
	ObfuscateHand(u8),

}

enum Prompt {
	PickColor,
	PickCardsFromHand(u8, u8),
	PickNumeral,
	PickPlayer,
	Approval,
}

