# Meme Finder Backend 🚀
![Pipeline](https://git.averyan.ru/meme-finder/backend/badges/main/pipeline.svg)

Backend for Meme Finder written in Rust

### Configuration (docker-compose):
```yaml
environment:
  MEILI_URL: http://meilisearch:7700 # url of meilisearch database
  MEILI_MASTER_KEY: key # meilisearch api key
  CORS_ORIGIN: https://memefinder.ru # additional allowed cors origin
  IMAGES_DIR: /data/images # directory for persistance image storage
```

### API Requests:
```bash
# search images
curl 'https://memefinder.ru/api/images?q=amogus&limit=30'
# remove image
curl -X DELETE 'https://memefinder.ru/api/images/14bcdf8f-edf7-4bdf-8f9f-1e6a248d9737'
```

### Related projects:
  - [Rust](https://www.rust-lang.org/) 🚀 - fast memory safe programming language
  - [MeiliSearch](https://www.meilisearch.com/) 🔎 - main text search database
  - [Actix Web](https://actix.rs/) 🌐 - rust web server framework
  - [Image-rs](https://github.com/image-rs/image) 🌄 - rust image library
