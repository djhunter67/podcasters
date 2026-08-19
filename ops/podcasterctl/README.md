# podcastersctl 
## podcastersctl initial state
```podcastersctl
│
├── doctor
│
├── dev
│   ├── up
│   ├── down
│   ├── status
│   └── reset
│
├── ci
│   ├── verify
│   └── integration
│
├── mongo
│   ├── check
│   ├── status
│   └── reconcile
│
├── redis
│   ├── check
│   └── status
│
├── config
│   ├── validate
│   └── show
│
├── smoke
│
├── deploy
│   ├── status
│   ├── staging
│   ├── production
│   └── rollback
│
├── kubernetes
│   ├── status
│   ├── pods
│   ├── nodes
│   └── events
│
├── backup
│   ├── create
│   ├── verify
│   └── restore
│
├── diagnostics
│   └── collect
│
└── version
```
## podcastersctl doctor

### Podcasters Environment

- Rust
  - ✓ nightly-2026-08-18
  - ✓ rustfmt
  - ✓ clippy
  - ✓ rustc-codegen-cranelift

- Docker
  - ✓ daemon reachable
  - ✓ current user may create containers

- MongoDB
  - ✓ mongodb://127.0.0.1:27017
  - ✓ ping
  - ✓ database: podcasters_test

- Redis
  - ✓ redis://127.0.0.1:6379
  - ✓ PONG

- Kubernetes
  - ✓ context: homelab
  - ✓ cluster reachable
  - ✓ nodes: 3/3 Ready

- Podcasters
  - ✓ backend configuration
  - ✓ frontend configuration
  - ✓ API health
  - ✓ frontend health

Overall: HEALTHY

## podcastersctl dev
podcastersctl dev up
podcastersctl dev down
podcastersctl dev status
podcastersctl dev reset

The command `dev up` could: 
	- Verify docker
	- start development MongoDB
	- Start development Redis
	- Wait for health checks
	- Verify settings
	- Start backend
	- Start frontend
	
	
## podcastersctl [database] <command>
podcastersctl mongo status
podcastersctl mongo check
podcastersctl mongo reconcile

`check` could: 
users
  ✓ email unique index
  ✓ created_at index

podcasts
  ✓ feed_url unique index
  ✓ title text index

episodes
  ✓ podcast_id index
  ✗ published_at index missing

podcastersctl mongo reconcile
- Creates whats missing
- This is akin to establishing migration before millions of documents are a reality
	
podcastersctl redis status
podcastersctl redis stats
podcastersctl redis keys --namespace session
podcastersctl redis clear --namespace integration-test

- Production-destructive commands should require explicit safeguards

Ex.
- podcastersctl redis clear --environment production

- Refusing destructive operation.

	- Use:
		-  --confirm-production

## podcastersctl ci verify

- Which will run:
format
clippy
workspace check
unit tests
integration tests
Mongo health
Redis health
backend smoke test
frontend smoke test

	- Then CI is not the only place where CI behavior exists
	
	
- On my workstation:
`podcastersctl ci verify`
- On your workstation:
`podcastersctl ci verify`
- On the self-hosted GitHub runner (before the CI from GitHub runs):
`podcastersctl ci verify`

## podcasterctl smoke
`podcasterctl smoke --environment staging`
- Initially check:
  - Backend `/health`
  - Frontend `/health`
  - Mongo connectivity through backend
  - Redis connectivity
  - API version
  - Build version
  
- Later:
  - ✓ frontend responding
  - ✓ backend responding
  - ✓ MongoDB reachable
  - ✓ Redis reachable
  - ✓ authentication round trip
  - ✓ podcast search
  - ✓ RSS ingestion worker
  - ✓ expected build 0.8.4

## podcasterctl diagnostics
`podcasterctl diagnostics collect`

- Ideally produce something like:
`podcasters-diagnostics-2026-08-19T113211Z.tar.gz`

- Containing non-secret information:
  - Podcasters version.
  - Git commit.
  - Rust version.
  - OS/kernel.
  - Architecture.
  - Kubernetes context name.
  - Kubernetes pod status.
  - deployment status.
  - recent Podcasters logs.
  - health-check output.
  - Mongo server version.
  - Redis server version.
  - configuration keys with values redacted.
  - resource usage.
  - networking diagnostics.
  
- Explicitly exclude:
  - passwords
  - tokens
  - connection-string credentials
  - cookies
  - signing keys
  
- When something fails later, instead of twenty diagnostic exchanges, start with:
`podcastersctl diagnostics collect`


## podcasterctl version
`podcasterctl version production`

#### Give every deployment an identity

- Example endpoint:
  `GET /v1/version`
  
  ```{
  "version": "0.1.0",
  "commit": "768bc15",
  "built_at": "2026-08-19T11:22:18Z",
  "rustc": "nightly-2026-08-18",
  "target": "x86_64-unknown-linux-gnu"
	  }```
	  
- This eliminates:
"Which build is actually running?"

## podcastersctl incident assess
`podcasterctl incident assess`

```Text
PRODUCTION ASSESSMENT

Backend
  6 desired
  4 healthy
  2 CrashLoopBackOff

Frontend
  4/4 healthy

MongoDB
  healthy
  latency 11ms

Redis
  healthy
  hit rate 93%

Feed workers
  queue depth: 187,122
  oldest job: 41m

Primary finding:
  feed-worker deployment is unhealthy

Suggested next command:
  podcastersctl kubernetes inspect feed-worker
  ```
	


## Milestone 0
`podcastersctl doctor
podcastersctl version
podcastersctl config validate
podcastersctl mongo check
podcastersctl redis check`

## Milestone 1
`podcastersctl dev up
podcastersctl dev down
podcastersctl ci verify
podcastersctl smoke`

## Milestone 2
`podcastersctl mongo reconcile
	podcastersctl diagnostics collect`
	
## Milestone 3
- After kubernetes deployment begins

`podcastersctl k8s status
podcastersctl deploy
podcastersctl rollback`

## Road Map
- Evaluate a Rust Kubernetes controller/operator after the deployment lifecycle is well understood.
Kubernetes custom controllers are particularly suited to encoding domain-specific operational
knowledge once that knowledge actually exists.

	
