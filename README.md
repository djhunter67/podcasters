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
  
