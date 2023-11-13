CREATE TABLE "PollutionReading" (
  "id" INTEGER GENERATED BY DEFAULT AS IDENTITY UNIQUE PRIMARY KEY NOT NULL,
  "timestamp" timestamp NOT NULL,
  "AQI" integer NOT NULL,
  "CO" float NOT NULL,
  "NO" float NOT NULL,
  "NO2" float NOT NULL,
  "O3" float NOT NULL,
  "SO2" float NOT NULL,
  "PM2_5" float NOT NULL,
  "NH3" float NOT NULL
);

CREATE TABLE "WeatherReading" (
  "id" INTEGER GENERATED BY DEFAULT AS IDENTITY UNIQUE PRIMARY KEY NOT NULL,
  "timestamp" timestamp NOT NULL,
  "condition" text NOT NULL,
  "description" text NOT NULL,
  "icon" text NOT NULL,
  "tempReal" float NOT NULL,
  "tempFeel" float NOT NULL,
  "pressureSea" integer NOT NULL,
  "humidity" integer NOT NULL,
  "pressureGround" integer NOT NULL,
  "visibility" integer NOT NULL,
  "windSpeed" float NOT NULL,
  "windDeg" integer NOT NULL,
  "windGust" float NOT NULL,
  "rain1H" float,
  "rain3H" float,
  "snow1H" float,
  "snow3H" float,
  "clouds" integer NOT NULL,
  "dt" integer NOT NULL,
  "sunrise" integer NOT NULL,
  "sunset" integer NOT NULL
);

CREATE TABLE "HomeSummary" (
  "id" INTEGER GENERATED BY DEFAULT AS IDENTITY UNIQUE PRIMARY KEY NOT NULL,
  "lastChanged" timestamp NOT NULL,
  "houseTemp" float,
  "houseHumidity" integer,
  "capability" integer NOT NULL,
  "systemActive" integer NOT NULL
);

CREATE TABLE "EnvCapability" (
  "id" INTEGER GENERATED BY DEFAULT AS IDENTITY UNIQUE PRIMARY KEY NOT NULL,
  "heating" boolean NOT NULL DEFAULT 'false',
  "cooling" boolean NOT NULL DEFAULT 'false',
  "lastChanged" datetime
);

CREATE TABLE "HVACactivity" (
  "id" INTEGER GENERATED BY DEFAULT AS IDENTITY UNIQUE PRIMARY KEY NOT NULL,
  "heating" boolean NOT NULL DEFAULT 'false',
  "heatLastChange" datetime,
  "cooling" boolean NOT NULL DEFAULT 'false',
  "coolLastChange" datetime
);

CREATE TABLE "Zones" (
  "id" INTEGER GENERATED BY DEFAULT AS IDENTITY UNIQUE PRIMARY KEY NOT NULL,
  "Name" text NOT NULL,
  "active" boolean NOT NULL,
  "capability" integer NOT NULL,
  "timeAdded" datetime NOT NULL,
  "lastChanged" datetime,
  "currentTemp" float,
  "currentHumid" integer,
  "systemActive" integer NOT NULL,
  "presence" boolean,
  "thresholdsClosed" boolean
);

CREATE TABLE "Schedules" (
  "id" INTEGER GENERATED BY DEFAULT AS IDENTITY UNIQUE PRIMARY KEY NOT NULL,
  "active" boolean NOT NULL,
  "name" text NOT NULL,
  "associatedZone" integer,
  "lastChanged" datetime,
  "timeStart" time,
  "timeEnd" time,
  "weekDay" integer,
  "dateStart" date,
  "dateEnd" date,
  "tempMin" float,
  "tempMax" float
);

CREATE TABLE "Sensors" (
  "id" INTEGER GENERATED BY DEFAULT AS IDENTITY UNIQUE PRIMARY KEY NOT NULL,
  "active" boolean NOT NULL,
  "Name" text NOT NULL,
  "Token" text UNIQUE NOT NULL,
  "associatedZone" integer,
  "timeAdded" timestamp NOT NULL,
  "timeUpdated" datetime,
  "comType" integer NOT NULL,
  "comLast" datetime,
  "currentTemp" float,
  "currentHumid" integer,
  "presence" boolean,
  "thresholdOpen" boolean
);

CREATE TABLE "Controllers" (
  "id" INTEGER GENERATED BY DEFAULT AS IDENTITY UNIQUE PRIMARY KEY NOT NULL,
  "Name" text UNIQUE NOT NULL,
  "active" boolean NOT NULL,
  "comType" integer NOT NULL,
  "Primary" boolean NOT NULL DEFAULT 'true',
  "associatedZone" integer,
  "Token" text UNIQUE NOT NULL,
  "timeAdded" timestamp NOT NULL,
  "timeChanged" datetime,
  "timeConnectLast" datetime,
  "capability" integer NOT NULL,
  "systemActive" integer NOT NULL
);

CREATE TABLE "Communication" (
  "id" INTEGER GENERATED BY DEFAULT AS IDENTITY UNIQUE PRIMARY KEY NOT NULL,
  "Name" text NOT NULL,
  "active" boolean NOT NULL
);

CREATE TABLE "Alerts" (
  "id" INTEGER GENERATED BY DEFAULT AS IDENTITY UNIQUE PRIMARY KEY NOT NULL,
  "Name" text UNIQUE NOT NULL,
  "active" boolean NOT NULL,
  "tripped" boolean NOT NULL,
  "comType" integer,
  "associatedSchedule" integer,
  "associatedZone" integer,
  "Actions" text
);

CREATE TABLE "Weekdays" (
  "id" INTEGER GENERATED BY DEFAULT AS IDENTITY UNIQUE PRIMARY KEY NOT NULL,
  "Sunday" boolean NOT NULL,
  "Monday" boolean NOT NULL,
  "Tuesday" boolean NOT NULL,
  "Wednesday" boolean NOT NULL,
  "Thursday" boolean NOT NULL,
  "Friday" boolean NOT NULL,
  "Saturday" boolean NOT NULL
);

CREATE TABLE "ManualChangeHistory" (
  "id" INTEGER GENERATED BY DEFAULT AS IDENTITY UNIQUE PRIMARY KEY NOT NULL,
  "changeTiming" timestamp NOT NULL,
  "changeWeather" integer NOT NULL,
  "changePollution" integer NOT NULL,
  "changeSource" integer NOT NULL,
  "newTemp" float,
  "newHumidity" integer,
  "changeSchedule" integer,
  "cancelledTiming" timestamp
);

CREATE TABLE "ChangeSource" (
  "id" INTEGER GENERATED BY DEFAULT AS IDENTITY UNIQUE PRIMARY KEY NOT NULL,
  "name" text UNIQUE NOT NULL
);

CREATE TABLE "SensorReadingHistory" (
  "id" INTEGER GENERATED BY DEFAULT AS IDENTITY UNIQUE PRIMARY KEY NOT NULL,
  "sensorID" integer NOT NULL,
  "timestamp" timestamp NOT NULL,
  "readingTemp" float,
  "readingHumidity" integer,
  "readingPresence" boolean,
  "readingThresholdOpen" boolean
);

COMMENT ON TABLE "PollutionReading" IS 'Stores API pollution responses';

COMMENT ON COLUMN "PollutionReading"."id" IS 'Consider UUID instead';

COMMENT ON TABLE "WeatherReading" IS 'Stores weather API responses';

COMMENT ON COLUMN "WeatherReading"."id" IS 'Consider UUID instead';

COMMENT ON COLUMN "WeatherReading"."dt" IS 'Time of calculation from the API';

COMMENT ON TABLE "HomeSummary" IS 'Whole-house summary.';

COMMENT ON TABLE "EnvCapability" IS 'Table to contain what a house/zone/controller CAN do';

COMMENT ON TABLE "HVACactivity" IS 'Table to contain what a house/zone/controller IS doing';

COMMENT ON TABLE "Zones" IS 'They do not inherently need a controller or sensors';

COMMENT ON COLUMN "Zones"."currentTemp" IS 'Needs to be the median temp of all sensors';

COMMENT ON COLUMN "Zones"."currentHumid" IS 'Needs to be the median humidity of all sensors';

COMMENT ON COLUMN "Zones"."presence" IS 'If any sensors sense presence, this is true';

COMMENT ON TABLE "Schedules" IS 'Schedules for desired temperature or alerts';

COMMENT ON TABLE "Sensors" IS 'Table for tracking sensors';

COMMENT ON COLUMN "Sensors"."Token" IS 'Associated token for authenticating to the API';

COMMENT ON COLUMN "Sensors"."comLast" IS 'Last time the server successfully pulled data from the sensor';

COMMENT ON TABLE "Controllers" IS 'Table for tracking controllers. Controllers can toggle heating and cooling systems physically.';

COMMENT ON COLUMN "Controllers"."Primary" IS 'If there are multiple controllers in a zone with the same capability, this one will be tried first and others are tried only after this one fails. If no primaries, all controllers are toggled at the same time.';

COMMENT ON COLUMN "Controllers"."Token" IS 'Associated token for authenticating to the API';

COMMENT ON COLUMN "Controllers"."timeConnectLast" IS 'Last time the server successfully changed a state on the controller.';

COMMENT ON TABLE "Communication" IS 'Table to contain valid ways for the server, controllers and sensors to talk to each other.';

COMMENT ON TABLE "Alerts" IS 'Table for tracking available alerts and what they do when tripped';

COMMENT ON TABLE "Weekdays" IS 'Table for days of the week. I might not need this.';

COMMENT ON TABLE "ManualChangeHistory" IS 'Table to capture times when a manual change is made to help build patterns';

COMMENT ON COLUMN "ManualChangeHistory"."id" IS 'Consider UUID instead';

COMMENT ON TABLE "ChangeSource" IS 'List of available spots to make changes in the application';

COMMENT ON TABLE "SensorReadingHistory" IS 'History of all sensor readings';

COMMENT ON COLUMN "SensorReadingHistory"."id" IS 'Consider UUID instead';

ALTER TABLE "HomeSummary" ADD FOREIGN KEY ("capability") REFERENCES "EnvCapability" ("id");

ALTER TABLE "HomeSummary" ADD FOREIGN KEY ("systemActive") REFERENCES "HVACactivity" ("id");

ALTER TABLE "Zones" ADD FOREIGN KEY ("capability") REFERENCES "EnvCapability" ("id");

ALTER TABLE "Zones" ADD FOREIGN KEY ("systemActive") REFERENCES "HVACactivity" ("id");

ALTER TABLE "Schedules" ADD FOREIGN KEY ("associatedZone") REFERENCES "Zones" ("id");

ALTER TABLE "Schedules" ADD FOREIGN KEY ("weekDay") REFERENCES "Weekdays" ("id");

ALTER TABLE "Sensors" ADD FOREIGN KEY ("associatedZone") REFERENCES "Zones" ("id");

ALTER TABLE "Sensors" ADD FOREIGN KEY ("comType") REFERENCES "Communication" ("id");

ALTER TABLE "Controllers" ADD FOREIGN KEY ("comType") REFERENCES "Communication" ("id");

ALTER TABLE "Controllers" ADD FOREIGN KEY ("associatedZone") REFERENCES "Zones" ("id");

ALTER TABLE "Controllers" ADD FOREIGN KEY ("capability") REFERENCES "EnvCapability" ("id");

ALTER TABLE "Controllers" ADD FOREIGN KEY ("systemActive") REFERENCES "HVACactivity" ("id");

ALTER TABLE "Alerts" ADD FOREIGN KEY ("comType") REFERENCES "Communication" ("id");

ALTER TABLE "Alerts" ADD FOREIGN KEY ("associatedSchedule") REFERENCES "Schedules" ("id");

ALTER TABLE "Alerts" ADD FOREIGN KEY ("associatedZone") REFERENCES "Zones" ("id");

ALTER TABLE "ManualChangeHistory" ADD FOREIGN KEY ("changeWeather") REFERENCES "WeatherReading" ("id");

ALTER TABLE "ManualChangeHistory" ADD FOREIGN KEY ("changePollution") REFERENCES "PollutionReading" ("id");

ALTER TABLE "ManualChangeHistory" ADD FOREIGN KEY ("changeSource") REFERENCES "ChangeSource" ("id");

ALTER TABLE "ManualChangeHistory" ADD FOREIGN KEY ("changeSchedule") REFERENCES "Schedules" ("id");

ALTER TABLE "SensorReadingHistory" ADD FOREIGN KEY ("sensorID") REFERENCES "Sensors" ("id");
