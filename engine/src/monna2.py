"""
Monna2 (tv.monna2.top) indexer adapter.
"""

import asyncio
import logging
import re

import httpx
from selectolax.parser import HTMLParser

from core.indexers.base import IndexerAdapter, IndexerResult

logger = logging.getLogger(__name__)


class Monna2Adapter(IndexerAdapter):
    """
    Adapter for tv.monna2.top (Russian movies/TV shows).

    Features:
    - Simple HTTP (no Cloudflare)
    - Both magnet links and torrent file downloads
    - Movies and TV shows
    - Russian content focus
    """

    def __init__(self, base_url: str = "https://tv.monna2.top", timeout: int = 30):
        super().__init__(base_url, timeout)
        # Create sync client
        self.client = httpx.Client(
            headers={"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"},
            timeout=timeout
        )
        # Semaphore will be created in async context
        self._semaphore = None

    @property
    def name(self) -> str:
        return "Monna2"

    def search(self, query: str, limit: int = 50) -> list[IndexerResult]:
        """
        Search for torrents on Monna2.

        Search URL: https://tv.monna2.top/index.php?do=search&subaction=search&story={query}
        """
        results = []
        search_url = f"{self.base_url}/index.php"
        params = {"do": "search", "subaction": "search", "story": query}

        try:
            response = self.client.get(search_url, params=params)
            response.raise_for_status()

            parser = HTMLParser(response.text)

            # Find all article links in search results
            # Pattern: <a href="/category/id-title-year.html">Title (Year)</a>
            article_links = parser.css('a[href*=".html"]')

            for link in article_links:
                if len(results) >= limit:
                    break

                href = link.attributes.get("href", "")
                if not href or href.startswith("javascript:"):
                    continue

                # Build full URL
                if href.startswith("/"):
                    detail_url = self.base_url + href
                elif href.startswith("http"):
                    detail_url = href
                else:
                    continue

                # Skip non-content pages
                if any(skip in detail_url for skip in ["xfsearch", "page/", "disklaimer"]):
                    continue

                # Fetch torrent details from the page
                torrent = self._fetch_torrent_details(detail_url)
                if torrent:
                    results.append(torrent)

        except Exception as e:
            logger.error(f"Monna2 search failed: {e}")

        return results

    def get_feed(self, page: int = 1, limit: int = 50) -> list[IndexerResult]:
        """
        Get latest torrents from Monna2 homepage or serial page.

        Feed URLs:
        - Main: https://tv.monna2.top/
        - Serials: https://tv.monna2.top/serial/
        - Page 2+: https://tv.monna2.top/page/2/
        """
        results = []

        if page == 1:
            feed_url = f"{self.base_url}/"
        else:
            feed_url = f"{self.base_url}/page/{page}/"

        try:
            response = self.client.get(feed_url)
            response.raise_for_status()

            parser = HTMLParser(response.text)

            # Find all article links
            article_links = parser.css('a[href*=".html"]')

            for link in article_links:
                if len(results) >= limit:
                    break

                href = link.attributes.get("href", "")
                if not href or href.startswith("javascript:"):
                    continue

                # Build full URL
                if href.startswith("/"):
                    detail_url = self.base_url + href
                elif href.startswith("http"):
                    detail_url = href
                else:
                    continue

                # Skip non-content pages
                if any(skip in detail_url for skip in ["xfsearch", "page/", "disklaimer"]):
                    continue

                # Fetch torrent details
                torrent = self._fetch_torrent_details(detail_url)
                if torrent:
                    results.append(torrent)

        except Exception as e:
            logger.error(f"Monna2 feed fetch failed: {e}")

        return results

    async def get_feed_async(self, page: int = 1, limit: int = 50) -> list[IndexerResult]:
        """
        Get latest torrents from Monna2 (ASYNC VERSION - MUCH FASTER!).
        Fetches all detail pages in parallel using aiohttp.
        """
        logger.info(f"get_feed_async called: page={page}, limit={limit}")

        # Create semaphore in async context (limit concurrent requests to avoid overwhelming server)
        if self._semaphore is None:
            self._semaphore = asyncio.Semaphore(50)  # Increased from 10 to 50!

        results = []

        if page == 1:
            feed_url = f"{self.base_url}/"
        else:
            feed_url = f"{self.base_url}/page/{page}/"

        logger.info(f"Fetching feed from: {feed_url}")
        try:
            # Fetch the feed page
            async with httpx.AsyncClient(timeout=self.timeout) as client:
                response = await client.get(feed_url)
                response.raise_for_status()
                html = response.text

            parser = HTMLParser(html)

            # Find main content area (avoid sidebars, headers, footers)
            # Look for the main article listing container
            main_content = parser.css("#dle-content")[0] if parser.css("#dle-content") else parser
            
            if isinstance(main_content, HTMLParser):
                article_containers = main_content.css("article, div[class*='short'], div[class*='post'], div[class*='Short'], div[class*='Post']")
            else:
                article_containers = main_content.css("article, div[class*='short'], div[class*='post'], div[class*='Short'], div[class*='Post']")
            
            detail_urls = []
            seen_urls = set()

            # If no article containers found, fall back to finding links with specific patterns
            if not article_containers:
                if isinstance(main_content, HTMLParser):
                    article_links = main_content.css('a[href*=".html"]')
                else:
                    article_links = main_content.css('a[href*=".html"]')
                    
                for link in article_links:
                    if len(detail_urls) >= limit:
                        break

                    href = link.attributes.get("href", "")
                    if not href or href.startswith("javascript:"):
                        continue

                    # Build full URL
                    if href.startswith("/"):
                        detail_url = self.base_url + href
                    elif href.startswith("http"):
                        detail_url = href
                    else:
                        continue

                    # Skip non-content pages and duplicates
                    if any(
                        skip in detail_url
                        for skip in ["xfsearch", "page/", "disklaimer", "user/", "tags/"]
                    ):
                        continue

                    # Only include URLs with category paths (boevik, drama, serial, etc.)
                    if not any(
                        cat in detail_url
                        for cat in [
                            "/boevik/",
                            "/drama/",
                            "/serial/",
                            "/triller/",
                            "/komediya/",
                            "/fantastika/",
                            "/uzhasy/",
                            "/dokumentalnyy/",
                            "/melodrama/",
                            "/priklucheniya/",
                            "/semeynyy/",
                            "/voennyy/",
                            "/istoriya/",
                            "/biografiya/",
                            "/sport/",
                        ]
                    ):
                        continue

                    if detail_url not in seen_urls:
                        seen_urls.add(detail_url)
                        detail_urls.append(detail_url)

            else:
                # Extract links from article containers
                for container in article_containers:
                    if len(detail_urls) >= limit:
                        break
                        
                    # Find links within this container
                    container_links = container.css('a[href*=".html"]')
                    
                    for link in container_links:
                        if len(detail_urls) >= limit:
                            break

                        href = link.attributes.get("href", "")
                        if not href or href.startswith("javascript:"):
                            continue

                        # Build full URL
                        if href.startswith("/"):
                            detail_url = self.base_url + href
                        elif href.startswith("http"):
                            detail_url = href
                        else:
                            continue

                        # Skip non-content pages and duplicates
                        if any(
                            skip in detail_url
                            for skip in ["xfsearch", "page/", "disklaimer", "user/", "tags/"]
                        ):
                            continue

                        # Only include URLs with category paths (boevik, drama, serial, etc.)
                        if not any(
                            cat in detail_url
                            for cat in [
                                "/boevik/",
                                "/drama/",
                                "/serial/",
                                "/triller/",
                                "/komediya/",
                                "/fantastika/",
                                "/uzhasy/",
                                "/dokumentalnyy/",
                                "/melodrama/",
                                "/priklucheniya/",
                                "/semeynyy/",
                                "/voennyy/",
                                "/istoriya/",
                                "/biografiya/",
                                "/sport/",
                            ]
                        ):
                            continue

                        if detail_url not in seen_urls:
                            seen_urls.add(detail_url)
                            detail_urls.append(detail_url)

            # Fetch all detail pages in parallel
            logger.info(
                f"Fetching {len(detail_urls)} Monna2 pages in parallel..."
            )

            # Create async client for detail fetches
            async with httpx.AsyncClient(
                timeout=self.timeout,
                limits=httpx.Limits(max_connections=100, max_keepalive_connections=50)
            ) as client:
                tasks = [self._fetch_torrent_details_async(url, client) for url in detail_urls]
                torrents = await asyncio.gather(*tasks, return_exceptions=True)

            # Filter out None and exceptions
            error_count = 0
            for i, torrent in enumerate(torrents):
                if isinstance(torrent, IndexerResult):
                    results.append(torrent)
                elif isinstance(torrent, Exception):
                    error_count += 1
                    logger.error(
                        f"Error fetching {detail_urls[i]}: {type(torrent).__name__}: {torrent}"
                    )
                elif torrent is None:
                    error_count += 1
                    logger.debug(f"No result for {detail_urls[i]}")

            logger.info(
                f"Successfully fetched {len(results)} torrents from Monna2 ({error_count} errors)"
            )

        except Exception as e:
            logger.error(f"Monna2 async feed fetch failed: {e}")

        return results

    async def _fetch_torrent_details_async(
        self, detail_url: str, client: httpx.AsyncClient
    ) -> IndexerResult | None:
        """Fetch torrent details asynchronously with rate limiting."""
        async with self._semaphore:  # Limit concurrent requests
            try:
                response = await client.get(
                    detail_url,
                    headers={
                        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
                    },
                )
                response.raise_for_status()
                html = response.text

                parser = HTMLParser(html)

                # Extract title (same logic as sync version)
                title = None
                h1 = parser.css_first("h1")
                if h1:
                    title = h1.text(strip=True)
                    # Remove "скачать торрент" suffix if present
                    title = re.sub(r"\s+скачать\s+торрент.*$", "", title, flags=re.IGNORECASE)
                    
                if not title:
                    logger.debug(f"No title found for {detail_url}")
                    return None

                # Extract ALL quality options from quality table
                # The iframe is loaded dynamically, so we construct the URL ourselves
                magnet_link = None
                infohash = None
                quality_options = []
                
                # Build the quality table URL from the title
                from urllib.parse import quote
                from core.indexers.base import QualityOption
                
                quality_table_url = f"https://poisk.ww33.top/ser2.php?query={quote(title)}"
                
                try:
                    # Fetch the quality table (use async client)
                    table_response = await client.get(quality_table_url, timeout=5)
                    # Don't raise for 403 - the site still returns data
                    if table_response.status_code not in [200, 403]:
                        logger.warning(f"Quality table returned status {table_response.status_code}")
                        return None
                    
                    table_parser = HTMLParser(table_response.text)
                    
                    # Find all rows in the table
                    rows = table_parser.css('tbody tr')
                    logger.info(f"🔍 Found {len(rows)} quality options")
                    
                    for row in rows:
                        cells = row.css('td')
                        if len(cells) >= 6:
                            # Extract magnet link
                            magnet_elem = cells[1].css_first('a[href^="magnet:"]')
                            if not magnet_elem:
                                continue
                            
                            magnet = magnet_elem.attributes.get("href")
                            if not magnet:
                                continue
                            
                            # Extract infohash
                            opt_infohash = None
                            match = re.search(r"btih:([a-fA-F0-9]{40})", magnet)
                            if match:
                                opt_infohash = match.group(1).lower()
                            
                            # Extract quality/title and normalize it
                            quality_text = cells[3].text(strip=True)
                            
                            # Extract clean quality label (resolution + source)
                            quality_label = self._extract_quality_label(quality_text)
                            
                            # Extract size
                            size_text = cells[2].text(strip=True)
                            # Parse size bytes from the <u> tag
                            size_u = cells[2].css_first('u')
                            size_bytes = int(size_u.text(strip=True)) if size_u else None
                            
                            # Extract seeders/leechers
                            seeders = int(cells[4].text(strip=True)) if cells[4].text(strip=True).isdigit() else 0
                            leechers = int(cells[5].text(strip=True)) if cells[5].text(strip=True).isdigit() else 0
                            
                            quality_options.append(QualityOption(
                                quality=quality_label,  # Use normalized label
                                magnet_link=magnet,
                                infohash=opt_infohash,
                                size=size_text.split(str(size_bytes))[-1] if size_bytes else size_text,
                                size_bytes=size_bytes,
                                seeders=seeders,
                                leechers=leechers
                            ))
                    
                    # Sort quality options by seeders (descending) for best quality first
                    if quality_options:
                        quality_options.sort(key=lambda x: x.seeders, reverse=True)
                        magnet_link = quality_options[0].magnet_link
                        infohash = quality_options[0].infohash
                        logger.info(f"✅ Found {len(quality_options)} quality options for {title} (best: {quality_options[0].seeders} seeders)")
                except Exception as e:
                    logger.debug(f"⚠️ Failed to fetch quality table for {title}: {e}")

                # Extract torrent file download link
                torrent_file_url = None
                for link in parser.css("a[href]"):
                    href = link.attributes.get("href", "")
                    if "download" in href.lower() and not href.startswith("magnet:"):
                        if href.startswith("/"):
                            torrent_file_url = self.base_url + href
                        else:
                            torrent_file_url = href
                        break

                if not magnet_link and not torrent_file_url:
                    logger.debug(f"No download links found for {detail_url}")
                    return None

                # Extract category from URL
                category = None
                category_match = re.search(r"/([^/]+)/\d+-", detail_url)
                if category_match:
                    category = category_match.group(1)

                # Create quality_options array with basic data (fallback when external service fails)
                quality_options = []
                if magnet_link:
                    quality_options = [{
                        'quality': '1080p',  # Default quality
                        'size': None,  # Size unknown when external service fails
                        'size_bytes': None,
                        'seeders': seeders,
                        'leechers': leechers,
                        'infohash': infohash,
                        'magnet_link': magnet_link
                    }]
                    logger.debug(f"Created fallback quality_options without size data")

                # Create IndexerResult
                result = IndexerResult(
                    title=title,
                    infohash=infohash,
                    magnet_link=magnet_link,
                    torrent_file_url=torrent_file_url,
                    detail_url=detail_url,
                    category=category,
                    source="Monna2",
                    quality_options=quality_options,  # Pass the created array instead of None
                    # Use seeders from first quality option if available
                    seeders=quality_options[0]['seeders'] if quality_options else seeders,
                    leechers=leechers,
                )

                # Extract and store additional metadata
                # We'll add it to the dict when to_dict() is called
                metadata = self._extract_metadata(parser)
                # Store metadata as an attribute for later use
                result._monna2_metadata = metadata
                logger.info(
                    f"📦 Monna2 metadata for '{title}': {list(metadata.keys()) if metadata else 'EMPTY'}"
                )

                return result

            except Exception as e:
                logger.debug(f"Failed to fetch {detail_url}: {e}")
                return None

    def _fetch_torrent_details(self, detail_url: str) -> IndexerResult | None:
        """
        Fetch torrent details from a detail page.

        Extracts:
        - Title from page title or H1
        - Magnet link: <a href="magnet:?xt=urn:btih:...">
        - Torrent file: <a href="https://g.ww33.top/download/...">
        - Infohash from magnet link
        """
        try:
            response = self.client.get(detail_url)
            response.raise_for_status()

            parser = HTMLParser(response.text)

            # Extract title
            title = None
            h1 = parser.css_first("h1")
            if h1:
                title = h1.text(strip=True)
                # Remove "скачать торрент" suffix if present
                title = re.sub(r"\s+скачать\s+торрент.*$", "", title, flags=re.IGNORECASE)

            if not title:
                title_tag = parser.css_first("title")
                if title_tag:
                    title = title_tag.text(strip=True)
                    title = re.sub(r"\s+скачать\s+торрент.*$", "", title, flags=re.IGNORECASE)

            if not title:
                logger.debug(f"No title found for {detail_url}")
                return None

            # Extract magnet link
            magnet_link = None
            infohash = None
            magnet_elem = parser.css_first('a[href^="magnet:"]')
            if magnet_elem:
                magnet_link = magnet_elem.attributes.get("href")
                # Extract infohash from magnet
                match = re.search(r"btih:([a-fA-F0-9]{40})", magnet_link)
                if match:
                    infohash = match.group(1).lower()

            # Extract torrent file download link
            torrent_file_url = None
            # Pattern: <a href="https://g.ww33.top/download/..."> or similar
            # Look for any link with "download" in href
            for link in parser.css("a[href]"):
                href = link.attributes.get("href", "")
                if "download" in href.lower() and not href.startswith("magnet:"):
                    torrent_file_url = href
                    break

            # If no magnet or torrent file, skip
            if not magnet_link and not torrent_file_url:
                logger.debug(f"No download links found for {detail_url}")
                return None

            # Extract category from URL
            category = None
            category_match = re.search(r"/([^/]+)/\d+-", detail_url)
            if category_match:
                category = category_match.group(1)

            # Create IndexerResult
            result = IndexerResult(
                title=title,
                infohash=infohash,
                magnet_link=magnet_link,
                torrent_file_url=torrent_file_url,
                detail_url=detail_url,
                category=category,
                source="Monna2",
                # Note: Monna2 doesn't show seeders/leechers on the page
                # Would need to parse the torrent file or use DHT
                seeders=0,
                leechers=0,
            )

            # Extract and store additional metadata
            # We'll add it to the dict when to_dict() is called
            metadata = self._extract_metadata(parser)
            # Store metadata as an attribute for later use
            result._monna2_metadata = metadata
            logger.info(
                f"📦 Monna2 metadata for '{title}': {list(metadata.keys()) if metadata else 'EMPTY'}"
            )

            return result

        except Exception as e:
            logger.error(f"Failed to fetch details from {detail_url}: {e}", exc_info=True)
            return None

    def _extract_quality_label(self, torrent_name: str) -> str:
        """Extract clean quality label from torrent name using guessit."""
        from core.processing.guessit_helper import extract_quality_label
        return extract_quality_label(torrent_name)
    
    def _extract_metadata(self, parser: HTMLParser) -> dict:
        """
        Extract metadata from Monna2 detail page.

        Returns dict with: original_title, year, genres, director, cast, description
        """
        metadata = {}

        try:
            # Get the fullstory div
            fullstory = parser.css_first("div.fullstory")
            if not fullstory:
                return metadata

            text = fullstory.text()

            # Extract original title
            original_match = re.search(r"Оригинальное название:\s*(.+?)(?:\n|Год)", text)
            if original_match:
                metadata["monna2_original_title"] = original_match.group(1).strip()

            # Extract year (try both "Год выхода" and "Год выпуска")
            year_match = re.search(r"Год (?:выхода|выпуска):\s*(\d{4})", text)
            if year_match:
                metadata["monna2_year"] = int(year_match.group(1))

            # Extract genres - be very strict, only take until first newline or "Режиссёр"
            genre_match = re.search(
                r"Жанр:\s*([^\n<]+?)(?:<br>|Режиссёр|Режиссер|$)", text, re.IGNORECASE
            )
            if genre_match:
                genres_str = genre_match.group(1).strip()
                logger.info(f"   Extracted genres string: '{genres_str}'")
                # Split by comma and clean
                genres = [g.strip() for g in genres_str.split(",") if g.strip()]
                # Filter out obviously wrong entries (too long, contains colons, or looks like a name)
                # Names usually have capital letters in middle, genres are lowercase
                valid_genres = []
                for g in genres:
                    if (
                        len(g) < 30
                        and ":" not in g
                        and not re.search(r"[А-ЯA-Z][а-яa-z]+\s+[А-ЯA-Z]", g)
                    ):
                        valid_genres.append(g)
                metadata["monna2_genres"] = valid_genres[:5]  # Max 5 genres
                logger.info(f"   Valid genres: {valid_genres}")
            else:
                logger.debug("   No genre match found in text")

            # Extract director
            director_match = re.search(r"Режиссёр:\s*(.+?)(?:\n|В ролях)", text)
            if director_match:
                metadata["monna2_director"] = director_match.group(1).strip()

            # Extract cast (try both "В ролях" and "Актеры")
            cast_match = re.search(
                r"(?:В ролях|Актеры):\s*(.+?)(?:\n\n|Описание|О фильме)", text, re.DOTALL
            )
            if cast_match:
                cast_str = cast_match.group(1).strip()
                # Remove HTML links and clean
                cast_str = re.sub(r"<[^>]+>", "", cast_str)
                # Split by comma and clean
                metadata["monna2_cast"] = [
                    c.strip() for c in cast_str.split(",")[:10]
                ]  # First 10 actors

            # Extract runtime/duration
            runtime_match = re.search(r"Продолжительность:\s*(\d+):(\d+):(\d+)", text)
            if runtime_match:
                hours = int(runtime_match.group(1))
                minutes = int(runtime_match.group(2))
                total_minutes = hours * 60 + minutes
                metadata["monna2_runtime"] = total_minutes
                logger.info(f"   Extracted runtime: {total_minutes} minutes ({hours}h {minutes}m)")

            # Extract description (try both "О фильме" and "Описание")
            desc_match = re.search(
                r"(?:О фильме|Описание):?\s*(.+?)(?:Продолжительность|Файл|Скачать|$)",
                text,
                re.DOTALL,
            )
            if desc_match:
                description = desc_match.group(1).strip()
                # Clean up
                description = re.sub(r"\s+", " ", description)
                description = re.sub(r"<br>+", " ", description)  # Remove <br> tags
                metadata["monna2_description"] = description  # No limit - get full description

            # Also try meta description as fallback
            if "monna2_description" not in metadata:
                meta_desc = parser.css_first("meta[name='description']")
                if meta_desc:
                    metadata["monna2_description"] = meta_desc.attributes.get("content", "")[:500]

            # Extract poster image and Kinopoisk ID
            # Look for ALL images in fullstory
            if fullstory:
                all_imgs = fullstory.css("img")

                # Priority 1: Find uploads/posts images (these actually work!)
                for img in all_imgs:
                    src = img.attributes.get("src", "")
                    if "/uploads/posts/" in src:
                        if src.startswith("/"):
                            poster_src = f"https://tv.monna2.top{src}"
                        elif not src.startswith("http"):
                            poster_src = f"https://tv.monna2.top/{src}"
                        else:
                            poster_src = src
                        metadata["monna2_poster_url"] = poster_src
                        logger.info(f"   Found uploads/posts poster: {poster_src[:80]}...")
                        break

                # Extract Kinopoisk ID from any kinopoisk reference (for metadata, not poster)
                for img in all_imgs:
                    src = img.attributes.get("src", "")
                    kp_match = re.search(r"/kinopoisk/(\d+)", src)
                    if kp_match:
                        metadata["kinopoisk_id"] = kp_match.group(1)
                        metadata["kinopoisk_url"] = (
                            f"https://www.kinopoisk.ru/film/{kp_match.group(1)}/"
                        )
                        break

            # Priority 2: Try og:image if no uploads/posts found
            if "monna2_poster_url" not in metadata:
                og_image = parser.css_first("meta[property='og:image']")
                if og_image:
                    metadata["monna2_poster_url"] = og_image.attributes.get("content", "")
                    logger.info(f"   Using og:image: {metadata['monna2_poster_url'][:80]}...")

        except Exception as e:
            logger.debug(f"Failed to extract metadata: {e}")

        return metadata
