# Playback Controls — Edge Cases

Tracked edge cases for Seek, Queue, Skip, Shuffle, and Repeat.

## Seek
- [ ] Seek past duration → should clamp to end and trigger auto-advance
- [ ] Seek to negative → should clamp to 0
- [ ] Seek while paused → position updates but playback stays paused
- [ ] Seek on empty player (no track loaded) → should be a no-op
- [ ] Rapid successive seeks (user scrubbing fast) → debounce or let last one win?

## Queue
- [ ] Empty queue + Next/Prev → no-op, no crash
- [ ] Single-track queue + Next → behavior depends on repeat mode
- [ ] Queue with 0 valid file paths (deleted files) → skip gracefully
- [ ] User clicks same track that's already playing → restart or ignore?
- [ ] Playing from Playlist vs Album → both should populate the queue correctly

## Skip (Next / Previous)
- [ ] Next at end of queue (repeat off) → stop playback
- [ ] Next at end of queue (repeat all) → wrap to index 0
- [ ] Previous at start of queue → restart current track (no underflow)
- [ ] Previous within 3s → go to previous track
- [ ] Previous after 3s → restart current track
- [ ] Next/Prev during seek animation → should cancel seek and load new track

## Shuffle
- [ ] Toggle shuffle ON → current track stays, rest is shuffled
- [ ] Toggle shuffle OFF mid-queue → restore original order, keep current track
- [ ] Shuffle + Repeat All → after all shuffled tracks play, reshuffle for next cycle
- [ ] Shuffle with single-track queue → effectively a no-op
- [ ] Previous while shuffled → should go to previously played track (history-based), not the "previous index in shuffle order"

## Repeat
- [ ] Repeat One → auto-advance re-seeks to 0 on same track
- [ ] Repeat One + Next (manual) → should still advance to next track (user intent overrides)
- [ ] Repeat All + Shuffle → wrap around and reshuffle
- [ ] Repeat Off + last track → stop cleanly, reset UI

## Cross-Feature Interactions
- [ ] Shuffle ON + Repeat One → Repeat One takes priority (loop current track)
- [ ] Seek to end while Repeat One → should restart, not advance
- [ ] Load new queue while a track is playing → stop current, start new queue
- [ ] Factory reset while playing → stop playback, clear queue
