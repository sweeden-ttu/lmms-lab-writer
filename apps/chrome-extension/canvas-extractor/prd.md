# Untitled Project

## Project Overview



## User Stories

### US-1: Read and validate Canvas payload file

**Priority**: 1
**Status**: pending

**User Story**: 

#### Acceptance Criteria

- [ ] 1. WHEN the script is executed, THE system SHALL attempt to read /tmp/course_payload.json
- [ ] 2. IF the file does not exist, THEN THE system SHALL log an error message and exit with code 1
- [ ] 3. IF the file contains invalid JSON, THEN THE system SHALL log a parsing error and exit with code 1
- [ ] 4. WHEN the file is successfully read, THE system SHALL log the number of courses or modules found

### US-2: Download course module files

**Priority**: 1
**Status**: pending

**User Story**: 

#### Acceptance Criteria

- [ ] 1. WHEN the payload is parsed, THE system SHALL identify all file download links in course modules
- [ ] 2. WHEN a file link is found, THE system SHALL log the filename and download URL
- [ ] 3. WHEN downloading a file, THE system SHALL save it to a ./downloads directory organized by course/module
- [ ] 4. IF a download fails, THEN THE system SHALL log the error and continue with remaining files
- [ ] 5. WHEN a file is successfully downloaded, THE system SHALL log the file path and size
- [ ] 6. IF the downloads directory does not exist, THEN THE system SHALL create it

### US-3: Extract and display grades

**Priority**: 1
**Status**: pending

**User Story**: 

#### Acceptance Criteria

- [ ] 1. WHEN the payload is parsed, THE system SHALL identify all grade entries
- [ ] 2. WHEN grades are found, THE system SHALL extract assignment name, score, and total points
- [ ] 3. WHEN grade extraction is complete, THE system SHALL log a summary of all grades in a readable format
- [ ] 4. IF no grades are found, THEN THE system SHALL log a message indicating no grades were detected
- [ ] 5. WHEN grades are extracted, THE system SHALL save them to ./output/grades.json

### US-4: Extract and list video links

**Priority**: 1
**Status**: pending

**User Story**: 

#### Acceptance Criteria

- [ ] 1. WHEN the payload is parsed, THE system SHALL identify all video links (YouTube, Vimeo, mp4, embedded videos)
- [ ] 2. WHEN video links are found, THE system SHALL extract the URL and associated title or context
- [ ] 3. WHEN video extraction is complete, THE system SHALL log all video URLs with descriptions
- [ ] 4. WHEN videos are extracted, THE system SHALL save them to ./output/videos.json
- [ ] 5. IF no video links are found, THEN THE system SHALL log a message indicating no videos were detected

### US-5: Progress logging and error handling

**Priority**: 1
**Status**: pending

**User Story**: 

#### Acceptance Criteria

- [ ] 1. WHEN the script starts, THE system SHALL log a startup message with timestamp
- [ ] 2. WHEN each major operation begins, THE system SHALL log the operation name
- [ ] 3. WHEN each major operation completes, THE system SHALL log success/failure status and duration
- [ ] 4. IF any error occurs, THEN THE system SHALL log the error with context before continuing or exiting
- [ ] 5. WHEN the script completes, THE system SHALL log a summary including files downloaded, grades found, and videos extracted
- [ ] 6. WHEN the script completes successfully, THE system SHALL exit with code 0

### US-6: Command-line options

**Priority**: 2
**Status**: pending

**User Story**: 

#### Acceptance Criteria

- [ ] 1. WHEN the script is run with --help flag, THE system SHALL display usage information and exit
- [ ] 2. WHEN the script is run with --version flag, THE system SHALL display the version number and exit
- [ ] 3. WHEN the script is run with an invalid flag, THE system SHALL display an error and suggest --help

