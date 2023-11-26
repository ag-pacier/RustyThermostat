# Rusty Thermostat

## Web API Branch
This branch is being used to experiment with and develop the web API side of this. When in working order this will:

- Interact with the database backend to confirm active IDs for:
-- Zones
-- Controllers
-- Sensors
- Utilize authentication to validate:
-- Only allowed devices (controllers and sensors) can interact with the system
-- Only valid users can see see settings or conditions
- Facilitate:
-- Toggling controller's behavior
-- Adding/Removing/Changing Controllers
-- Adding/Removing/Changing Sensors
-- Reviewing sensor readings
-- Reviewing current settings for controllers and sensors
-- Adding/Removing/Changing Zones

What this is NOT is:
- A user friendly web interface

## From Main
Statement: Create a server application using primarily Rust that runs on Linux which fetches and stores weather information, in-home stats and controls heating and cooling "intelligently" EG don't turn on the heat if it's hot outside, don't turn on the AC if it is cold outside, allow for scheduling of control etc.

Needed:
- Open Weather API fetching
-- Determine frequency of updates
-- Determine what information is relevant and available
-- Plan long term storage of certain information to build trends

- Postgres Database storage
-- Learn basic management and data techniques
-- Understand sizing
-- Build flexible data structures

- In home stats
-- Design HTTP/S interface for sensors using TCP/IP
-- Design RS-485 interface for sensors using Serial
--- Best case scenario for RS-485 is using thermostat wire to communicate

- Climate control
-- Research HVAC systems and recommended functions
-- Fail-open functionality (Don't let it stay on if the software breaks)

- Security requirements/thoughts
-- Communication between application and database must be encrypted and authenticated
-- Communication between application and other servers must be encrypted (EG API fetching)
-- Communication between application and sensors over TCP/IP must be one-way (sensor sending data to application) if not encrypted
-- Communication between application and sensors over RS-485 must be authenticated application-side
