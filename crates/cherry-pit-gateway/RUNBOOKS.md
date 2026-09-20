# Stale-lock recovery

Source contract: [gh-report CHE-0047:R5](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/cherry/CHE-0047-operational-recovery-runbooks.md).

Before deleting a lock sentinel or forcing failover, record the filesystem,
process identity and ownership evidence. Advisory locks are normally released
when the owning file descriptor closes; an existing sentinel alone does not
prove that its owner is dead.

1. Identify the affected store and inspect which process holds its lock.
2. Establish the process identity and whether it is still active.
3. Capture the sentinel path, modification time and size with
   `stale_lock_evidence`, and record the separate ownership evidence.
4. Apply the owning persistence backend's recovery procedure only after
   establishing that recovery will not disrupt a live owner.

`stale_lock_evidence` reads filesystem metadata. It does not determine the
lock-holder PID, prove that a lock is stale, or perform recovery.
