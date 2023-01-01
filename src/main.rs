use futures::future;
use futures::stream::{self, Stream, StreamExt};
use scraper::{Html, Selector};

const ARCHIVE_HOST: &str = "https://archive.org";

const ARCHIVE_COLLECTION_PAGE_BODY_EMPTY: &str =
    "<div class=\"no-results\">No results matched your criteria.</div>";

const ARCHIVE_COLLECTIOM: &str = "xboxcds";

pub fn stream_incremental(init: u64) -> impl Stream<Item = u64> {
    stream::unfold(init, |current| future::ready(Some((current, current + 1))))
}

async fn get_archive_collection_page(collection: String, page: u64) -> Option<String> {
    let uri = format!(
        "{}/details/{}?&scroll=1&sort=titleSorter&page={}",
        ARCHIVE_HOST, collection, page
    );

    let body = surf::get(&uri)
        .recv_string()
        .await
        .unwrap_or_else(|err| panic!("Failed to get collection page: {}, {}", &uri, err));

    if body == ARCHIVE_COLLECTION_PAGE_BODY_EMPTY {
        None
    } else {
        Some(body)
    }
}

fn parse_archive_collection_page(html: String) -> Vec<String> {
    let fragment = Html::parse_fragment(&html);
    let selector = Selector::parse(".item-ttl a").unwrap();

    fragment
        .select(&selector)
        .into_iter()
        .flat_map(|element| element.value().attr("href"))
        .map(|path| format!("{}{}", ARCHIVE_HOST, path))
        .collect::<Vec<String>>()
}

async fn get_archive_item_page(uri: String) -> String {
    surf::get(&uri)
        .recv_string()
        .await
        .unwrap_or_else(|err| panic!("Failed to get item page: {}, {}", &uri, err))
}

fn parse_archive_item_page(html: String) -> String {
    let fragment = Html::parse_fragment(&html);
    let selector = Selector::parse(".item-download-options .format-group a").unwrap();

    fragment
        .select(&selector)
        .into_iter()
        .filter(|element| element.text().any(|text| text.contains("TORRENT")))
        .flat_map(|element| element.value().attr("href"))
        .map(|path| format!("{}{}", ARCHIVE_HOST, path))
        .collect::<Vec<String>>()
        .first()
        .expect("Failed to get item torrent")
        .to_owned()
}

#[async_std::main]
async fn main() {
    stream_incremental(1)
        .map(|page| get_archive_collection_page(ARCHIVE_COLLECTIOM.to_owned(), page))
        .buffered(10)
        .take_while(|body| future::ready(body.is_some()))
        .filter_map(future::ready)
        .map(parse_archive_collection_page)
        .flat_map(stream::iter)
        .map(get_archive_item_page)
        .buffered(10)
        .map(parse_archive_item_page)
        .for_each(|uri| {
            println!("{}", uri);
            future::ready(())
        })
        .await
}
