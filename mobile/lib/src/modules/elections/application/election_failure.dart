/// Falhas da eleição que exigem orientações distintas ao eleitor.
enum ElectionFailure {
  evidenceInvalid,
  alreadyVoted,
  registrationClosed,
  keyConflict,
  notEligible,
  votingEnded,
  votingNotStarted,
  unauthorized,
  temporarilyUnavailable,
  requestRejected,
  unknown,
}
