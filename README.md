# Ponchik 🍩

A ⚡ blazing fast ⚡ slack donut app clone written in Rust 🦀

Ponchik helps integrate members of your team. Add Ponchik to a channel, and it will randomly pair individuals in the channel to be introduced in DMs. 

## Docs

Ponchik is deployed as a set of Vercel Functions built on the community-driven [Vercel Runtime for Rust](https://github.com/vercel-community/rust). Vercel Function invokations (or a command line interface) can be used to invoke stages of the pairing cycle, including making the introduction, sending checkpoints, and closing.

Pairings and their statuses are recorded in a PostgreSQL database using the sqlx PostgreSQL driver.

The Slack API is used to send Block Kit messages and handle user interaction.

## Todos
- [ ] Backend
    - [x] Ensure sqlx and postgres works in the vercel runtime
    - [x] Write pairings to database at intros generation
    - [x] Read active pairings from db and send midpoint checkins
    - [x] update db from user interactions
    - [ ] update db at pair close
    - [x] if sqlx and postgresql doesnt work then im either turning this into some middlewware with a js implementation powering the db connection, or writing a rust database library for google sheets
- [ ] Application 
    - [ ] Update pairing algo to prevent duplicate pairings
    - [ ] Update intro_launch endpoint to accept arguments for group size
    - [ ] implement a "nobody responded" interaction handling
    - [ ] implement a random group leader picker ("take point on making this meeting happen")
    - [ ] start a requested feature list. potentially include block lists, meeting schedule adjuster, multiple midpoint checkins,
    - [ ] Implement tracing 
    - [ ] set up jaegar / frontend for logs 
- [ ] Front end 
    - [ ] Create slack-authenticated frontend which displays db contents and can be used to interact with app
- [ ] Misc
    - [ ] lint everything, get rid of sheets stuff, probably move MeetingStatus struct to another module

### DB Table
group channel id | meeting status | date of intro | names...

meeting_status = {open, scheduled, closed_met, closed_no, closed_scheduled, closed_stale}
