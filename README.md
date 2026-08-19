# PODCASTERS -> podcast RS

A fully feautured podcasting app written in Rust and is blazingly fast.
This application Will feature Podcasting 2.0 support.


## Features to implement
- Smart speedups -> Speed up the playing instance without distortion.
- Boost the Voices -> Volume normalization across all podcasts.
- Chapters, transcripts, soundbites - Create a unique and enhanced podcast experience.
- Cross-platform - Android, iOS, and Web

## TODO

#### Early stages
- [ ] Research domain name and competitors
- [X] Purchase a domain name -> podcasters.com
- [ ] Resarch how to create an iOS app using Rust
- [ ] Can I use one source of code for all platforms? Tuari?
- [ ] Design the mobile interface
- [ ] Design the web interface
- [ ] Hello, world! on Android
- [ ] Hello, world! on iOS
- [ ] Hello, world! on the web application
- [ ] Database[s]
- [ ] Backend started
- [ ] Frontend started
- [ ] Self-host initial launch
- [ ] AI integration in a way that makes sense.


## Architecture

### Database Architecture
- users
- auth_sessions
- devices
- podcasts
- episodes
- subscriptions
- playback_states
- playlists
- bookmarks
- search_metadata
- feed_sources
- billing_customers
- entitlements
- Podcast metadata and user data should not be combined.
  - For example:
	- podcasts is global.
	- subscriptions associates users with podcasts.
	- episodes is global.
	- playback_states associates users with episodes.
- That prevents enormous duplication.

### Cache Layer
- Podcast metadata cache.
- Episode metadata cache.
- Search result cache.
- Session/authorization cache where appropriate.
- Rate-limit counters.
- Short-lived authentication state.
- Distributed locks for feed refresh.
- Job coordination.
	
#### Example 
- 10,000 User request the same podcast.
  - We should not perform 10,000 Database queries for 10,000 feed fetches
  
### IOS Developer Responsibilities
- AVFoundation/audio integration.
- Background playback.
- Now Playing.
- Remote controls.
- Notifications.
- Secure storage.
- iOS-specific Tauri plugins.
- Signing/provisioning.
- TestFlight/App Store.
- Swift only where the Apple APIs require it.

	- Do not rebuild our application twice

### Android Developer Responsibilities
- Android lifecycle integration.
- Background audio.
- MediaSession integration.
- Bluetooth controls.
- Notifications.
- Files/downloads.
- Android-specific Tauri plugins.
- Store publishing.
- Performance testing.
- Native Kotlin only where required.

	- Do no rebuild the application twice

### Podcast Ingestion

#### Phase 1
- We begin with third-party indexing/discovery.
- I would separate discovery from feed parsing.
- Discovery answers:
  - “Where is the podcast?”
- Feed parsing answers:
  - “What does the publisher's feed actually contain?”
- The RSS feed should remain authoritative whenever practical.
- For parsing, feed-rs currently provides a unified Rust parser for RSS, Atom, and JSON Feed, making it a reasonable candidate for the generic syndication layer.
- But Podcasting 2.0 should be handled by our own podcasting abstraction, not exposed directly through whatever RSS crate we choose.

#### Phase 2
- Build our normalized catalog.
- External index:
  - ↓
- Feed URL discovered:
  - ↓
- Fetch RSS:
  - ↓
- Parse:
  - ↓
- Normalize:
  - ↓
- Store Database:
  - ↓
- Cache layer:
  - ↓
- Search index/API
- At this stage Podcasters owns its internal representation of podcasts even though discovery still depends on outside services.

#### Phase 3
- Build our own crawler.
- New worker responsibilities:
- Feed scheduling.
- Conditional HTTP requests.
- Redirect handling.
- Dead-feed detection.
- Backoff.
- Feed deduplication.
- GUID reconciliation.
- Publisher rate limiting.
- Feed health.
- Podcast discovery.
- The crawler should be a separate binary:
  - cargo run -p backend --bin crawler
- But I would not create that binary yet.
- Build it when Phase 2 proves the data model.

### Podcasting 2.0
- This will be first-class from the beginning.
- Podcasting 2.0 currently defines namespace functionality such as chapters, transcripts, people, soundbites, live items, and other extensions.
- Our podcasting crate should therefore model:
- Traditional RSS.
- Podcast Namespace extensions.
- Unknown namespace elements.
- Important rule: never throw away metadata merely because our UI doesn't support it yet.
- That gives us forward compatibility.

### Audio Subsystem
- Audio processing stays on-device first.
- audio should define something conceptually similar to:
  - AudioProcessor
  - SpeedProcessor
  - LoudnessNormalizer
- Implementations could eventually differ:
  - audio/core
	- Algorithmic Rust.
   - Android adapter.
   - iOS adapter.
   - Browser/WASM adapter.
- Backend processing remains our fallback, not the primary design.

### Playback synchronization
- Every playing episode gets a persisted state containing approximately:
  - User ID.
  - Episode ID.
  - Position.
  - Duration.
  - Playback speed.
  - Completed state.
  - Device ID.
  - Updated timestamp/version.
- Clients should periodically checkpoint.
- Do not send an API request every second.
- Save after events such as:
  - Pause.
  - Seek.
  - App background.
  - Track change.
  - Periodic interval.
  - Application exit where possible.
  
### Conflict Resolution
- User listens on phone.
- Then opens web.
- Both may contain playback updates.
- We need an explicit reconciliation policy.
- Initial policy:
  - Server version/revision number.
  - Timestamp.
  - Device ID.
  - Most recent valid state wins.
- Later we can improve that for offline playback.

### Authentication architecture
- Keep the existing self-rolled authentication system.
- Refactor it behind an authentication service rather than replacing it.
- Target authentication roadmap:
  - Existing username/password.
  - Email addresses.
  - Email verification.
  - Password reset.
  - Passkeys/WebAuthn.
  - TOTP 2FA.
  - Recovery codes.
  - OAuth/OIDC.
  - Device/session management.
- Authorization remains conceptually separate from authentication.
- Authentication:
  - “Who are you?”
- Authorization:
  - “What may you do?”


### Session and devices
- Each login should eventually become an identifiable session.
- User should be able to see:
  - Arch Linux -- Chrome
  - Android -- Pixel
  - iOS -- iPhone
- And revoke individual sessions.
- That becomes especially important for a synchronized podcast application.

### Billing architecture
- First release
  - Billing may exist architecturally
  - Premium gates may contain zero production features
- A paid Supporter subscription is a legitament first monetization experiment:
  - Same functionality
  - Users financially support development
  - Optional supporter badge, if desired.
  
- Later, features that produce meaningful recurring infrastructure costs could naturally become premium candidates:
  - GPU AI processing
  - Large transcript storage
  - Advanced AI search
  - Cloud generated summaries or transcripts
- We can decide that from actual cost and user behavior rather than guessing now.

### Billing abstraction
- `billing` should normalize providers into our own states.
- Providers:
  - Stripe
  - Apple
  - Google
- Podcasters should understand:
  - `Plan`
  - `Subscription`
  - `Entitlement`
  - `BillingStatus`
- Podcasters business logic should not ask: 
  - "Did Stripe say customer X has price_123?"
- It should ask:
  - "Does user X have entitlement Y?"
  
### Search 
- Initial search:
  - External podcast discovery API
  - Database metadata
  - Cache Layer
- Later:
  - Dedicate full-text search engine if required
- Do not introduce Elasticserach/OpenSearch/Meilisearch on day one.
- We should first establish
  - Actual podcast count
  - Actual query volume
  - Search quality requirements
  
  

### AI
- AI should not be added simply so the application can advert 'AI.'
- Good future candidates
  - Episode summaries
  - Transcript semantic search
  - "Where did they discuss Rust?"
  - Chapter suggestions
  - Topic extractions
  - Podcast Recommendations
  - Question answering over an episode
- The multi-GPU Arch workstation could eventually become useful as a development/inference environment, but production architecture should not require that machine.
- AI should sit behind an abstraction such as:
  - `AiProvider`
  - `TranscriptionProvider`
  - `EmbeddingProvider`
- That lets us run:
  - Local models
  - Hosted models
  - GPU workers
  
### Open-source architecture
- Repository
  - Application source
  - Tests
  - Infrastructure templates
  - Database migration/index configuration
  - Example configuration
- Never commit:
  - Database production URI
  - Cache layer credentials
  - Stripe Keys
  - JWT/private signing keys
  - Apple signing credentials
  - OAuth secrets
  - API secrets
- Provide:
  - `.env.example`
- Never:
  - `.env`

### Application configuration
- Sections:
  - Server.
  - MongoDB.
  - Redis.
  - Authentication.
  - Billing.
  - External podcast index.
  - Email.
  - AI.
  - Logging.
- Validate configuration during startup.
- Fail immediately when required production configuration is missing.

### Observability
- Standardize on Rust `tracing`
- Every request gets:
  - Request ID.
  - Structured logs.
  - Duration.
  - Status.
- Eventually collect:
  - Request latency.
  - API error rates.
  - Feed refresh failures.
  - Cache hit rate.
  - MongoDB latency.
  - Worker queue depth.
  - Playback sync failures.
  - Authentication failures.
  
### Testing
- Unit tests
  - Domain logic.
  - Feed parsing.
  - Podcasting 2.0.
  - Authentication.
  - Billing state.
  - Playback reconciliation.
- Integration tests
  - Actix endpoints.
  - MongoDB repositories.
  - Redis cache.
  - Auth middleware.
- Feed fixtures
  - These will become extremely important.
  - Maintain real-world sanitized RSS samples demonstrating:
	- Basic RSS.
	- Malformed RSS.
	- Podcasting 2.0.
	- Chapters.
	- Transcripts.
	- Soundbites.
	- Redirects.
	- Missing GUIDs.
- Actix provides application testing helpers specifically for testing handlers, extractors, middleware, and applications.
### CI/CD
- Every pull request:
  - cargo fmt --check
  - cargo clippy
  - cargo test --workspace
  - Build backend.
  - Build frontend.
  - Dependency/security audit.
  - WASM frontend build.
- Main branch:
  - Build deployment artifact/container.
  - Integration tests.
  - Deploy staging.
  - Production deployment after defined promotion.
- Android/iOS pipelines come later.

## Short-term engineering goals 
### Milestone 0
- Architecture foundation
- [ ] Finalize workspace.
- [ ] Introduce shared.
- [ ] Introduce podcasting.
- [ ] Introduce audio.
- [ ] Configure workspace dependencies.
- [ ] Establish coding conventions.
- [ ] Establish CI.
- Definition of done
  - cargo test --workspace works from repository root.
  - Every crate builds.
  - CI is green.
  
### Milestone 0.5 - Engineering Platform Foundation
- [ ] Create ops/podcastersctl crate
- [ ] Implement doctor
- [ ] Implement config validate
- [ ] Implement Mongo health check
- [ ] Implement Redis health check
- [ ] Implement build/version metadata
- [ ] Have CI execute podcastersctl doctor
- [ ] Establish tracing field conventions
- [ ] Establish OpenTelemetry boundary

### Milestone 1 - Backend foundation
- Complete:
  - [ ] Actix application state.
  - [ ] Mongo connection.
  - [ ] Redis connection.
  - [ ] Error model.
  - [ ] API version routing.
  - [ ] Authentication integration.
  - [ ] Health endpoint.
- First useful endpoint:
  - `GET /api/v1/health`
- Then:
  - `/auth`
  - `/me`
  
### Milestone 2 - Podcast data
- Implement:
  - [ ] Podcast model.
  - [ ] Episode model.
  - [ ] Feed parser abstraction.
  - [ ] RSS retrieval.
  - [ ] Podcasting 2.0 parsing.
  - [ ] Mongo persistence.
- Goal:
  - Given a feed URL, Podcasters can ingest it and return normalized podcast + episode JSON.

### Milestone 3 - Web frontend
- Build:
  - [ ] Shell/navigation.
  - [ ] Authentication.
  - [ ] Podcast page.
  - [ ] Episode list.
  - [ ] Library.
  - [ ] Basic search.
  - [ ] Audio player.
  
### Milestone 4 - First usable Podcasters release
- User can:
  - [ ] Register/login.
  - [ ] Search.
  - [ ] Subscribe.
  - [ ] View library.
  - [ ] View podcast.
  - [ ] Play episode.
  - [ ] Change playback speed.
  - [ ] Resume episode.
  - [ ] Unsubscribe.
- At this milestone, it is officially a podcast application instead of an architecture project.

### Milestone 5 - Podcasting 2.0
- Add UI for:
  - [ ] Chapters.
  - [ ] Transcripts.
  - [ ] People.
  - [ ] Soundbites.
- Then:
  - [ ] Additional namespace tags incrementally.
  
### Milestone 6 - Android
- [ ] Create Tauri Android application.
- [ ] Login.
- [ ] Library.
- [ ] Playback.
- [ ] Download.
- [ ] Background playback.
- [ ] Device media controls.
- [ ] Sync.
- Tauri's current mobile development workflow supports Android-specific dev commands and Android Studio integration when native troubleshooting is required.

### Milestone 7 - iOS
- [ ] Move same application to the MacBook.
- [ ] Generate/maintain Tauri iOS project.
- [ ] Build in Xcode.
- [ ] Implement iOS audio adapter.
- [ ] Background playback.
- [ ] Media controls.
- [ ] Downloads.
- [ ] TestFlight.
- [ ] App Store.

## Long-term
- Year-one direction
  - [ ] podcasterctl command line Dev-ops tool
  - [ ] Excellent podcast playback.
  - [ ] Reliable synchronization.
  - [ ] Podcasting 2.0 leadership.
  - [ ] Android/iOS parity.
  - [ ] Strong search.
  - [ ] Large normalized podcast catalog.
  - [ ] Production observability.
  - [ ] Billing ready.
  - [ ] Email/passkey/2FA authentication.
- Longer term
  - [ ] Own crawler.
  - [ ] Independent podcast index.
  - [ ] Advanced discovery.
  - [ ] Offline-first synchronization.
  - [ ] AI transcript search.
  - [ ] AI summaries.
  - [ ] Native CarPlay.
  - [ ] Android Auto.
  - [ ] Desktop Tauri builds if demand exists.
  - [ ] Creator-facing functionality if strategically justified.
  
## Engineering backlog epics
- EPIC-001: Workspace foundation
- EPIC-002: Authentication modernization
- EPIC-003: Podcast feed ingestion
- EPIC-004: Podcasting 2.0
- EPIC-005: Podcast catalog
- EPIC-006: Search and discovery
- EPIC-007: User library
- EPIC-008: Playback engine
- EPIC-009: Playback synchronization
- EPIC-010: HTMX and Actix web client
- EPIC-011: Audio normalization
- EPIC-012: Smart playback speed
- EPIC-013: Billing
- EPIC-014: Android
- EPIC-015: iOS
- EPIC-016: Notifications
- EPIC-017: Observability
- EPIC-018: Podcast crawler
- EPIC-019: AI platform

## Technical-debt rules
- No unwrap() in normal production request paths unless an invariant makes failure impossible and it is documented.
- No Mongo queries directly from Actix handlers.
- No business logic in templates/components.
- No billing-provider-specific logic outside billing.
- No JavaScript added simply because a tutorial uses JavaScript.
- No new infrastructure service without an identified requirement.
- No microservice extraction simply because a module becomes large.
- No platform-specific fork of shared business logic unless unavoidable.
- No secrets in Git.
- No breaking API modification after clients depend on an endpoint without API-version consideration.

