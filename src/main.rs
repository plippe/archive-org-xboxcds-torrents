mod chaining;

use futures::future;
use futures::stream::{self, Stream, StreamExt};

use crate::chaining::ChainingExt;

const ARCHIVE_LIST_PAGE_BODY_EMPTY: &str =
    "<div class=\"no-results\">No results matched your criteria.</div>";

async fn get_archive_list_page(page: u64) -> Option<String> {
    println!("getting page {}", page);

    let uri = format!(
        "https://archive.org/details/xboxcds?&scroll=1&sort=titleSorter&page={}",
        page
    );

    let body = surf::get(uri)
        .recv_string()
        .await
        .unwrap_or_else(|_| "".to_owned());

    if body == ARCHIVE_LIST_PAGE_BODY_EMPTY {
        None
    } else {
        Some(body)
    }
}

fn stream_incremental(init: u64) -> impl Stream<Item = u64> {
    stream::unfold(init, |current| {
        (current, current + 1).pipe(Some).pipe(future::ready)
    })
}

#[async_std::main]
async fn main() {
    println!("Hello, world!");

    let pages = stream_incremental(1)
        .map(get_archive_list_page)
        .buffered(10)
        .take_while(|body| body.is_some().pipe(future::ready))
        .collect::<Vec<Option<String>>>()
        .await;

    println!("Pages {}", pages.len());
}
