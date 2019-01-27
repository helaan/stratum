pub enum JudgementStatus {
    // Transient states (0 -- 99)
    Queued = 0,
    InProgress = 1,

    // Permanent success states (100 -- 109)
    Accepted = 100,

    // Permanent user failure states (110 -- 199)
    WrongAnswer = 110,

    // Permament system failure states (200 and up)
    JudgingError = 200
}
