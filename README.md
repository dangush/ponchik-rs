# Ponchik Donut App

A blazing fast slack donut app clone written in Rust.

## Functionality

When ran, Ponchik will generate a list of pairs of members of a slack channel. It will make those introductions in Slack DMs, and record the pairs on a Google Sheet. 

## Lifecycle

- Monday send out pairings
- Friday midpoint checkin
- Next Friday result survey

Bot checks a google sheet cell to check time, chooses one of 3 paths as a result. 

## Todos
- [ ] Backend
    - [x] Ensure sqlx and postgres works in the vercel runtime
    - [ ] Write pairings to database at intros generation
    - [ ] Read active pairings from db and send midpoint checkins
    - [ ] update db from user interactions
    - [ ] update db at pair close
    - [ ] if sqlx and postgress doesnt work then im either turning this into some middlewware with a js implementation powering the db connection, or writing a rust database library for google sheets
- [ ] Application 
    - [ ] Update pairing algo to prevent duplicate pairings
    - [ ] Update intro_launch endpoint to accept arguments for group size
    - [ ] start a requested feature list. potentially include block lists, meeting schedule adjuster, multiple midpoint checkins,
    - [ ] Implement tracing 
    - [ ] set up jaegar / frontend for logs 
- [ ] Front end 
    - [ ] Create slack-authenticated frontend which displays db contents and can be used to interact with app

### DB Table
group channel id | meeting status | date of intro | names...

meeting_status = {open, scheduled, closed_met, closed_no, closed_scheduled, closed_stale}
