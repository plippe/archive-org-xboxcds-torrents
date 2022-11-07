mod chaining;

use futures::future;
use futures::stream::{self, Stream, StreamExt};
use scraper::{Html, Selector};

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

fn parse_archive_list_page(html: String) -> Vec<String> {
    let fragment = Html::parse_fragment(&html);
    let selector = Selector::parse(".item-ttl a").unwrap();

    fragment
        .select(&selector)
        .into_iter()
        .flat_map(|element| element.value().attr("href"))
        .map(|str| str.to_string())
        .collect::<Vec<String>>()
}

fn stream_incremental(init: u64) -> impl Stream<Item = u64> {
    stream::unfold(init, |current| {
        (current, current + 1).pipe(Some).pipe(future::ready)
    })
}

fn option_to_stream<A>(option: Option<A>) -> impl Stream<Item = A> {
    option.map_or(vec![], |some| vec![some]).pipe(stream::iter)
}

#[async_std::main]
async fn main() {
    println!("Hello, world!");

    let links = stream_incremental(1)
        .map(get_archive_list_page)
        .buffered(10)
        .take_while(|body| body.is_some().pipe(future::ready))
        .flat_map(option_to_stream)
        .map(parse_archive_list_page)
        .flat_map(stream::iter)
        .collect::<Vec<String>>()
        .await;

    println!("Links {}", links.len());
    println!("- {}", links[0]);
    println!("- {}", links[1]);
    println!("- {}", links[2]);
}
