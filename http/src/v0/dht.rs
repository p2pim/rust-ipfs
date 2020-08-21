use crate::v0::support::{with_ipfs, MaybeTimeoutExt, StringError, StringSerialized};
use ipfs::{Ipfs, IpfsTypes, PeerId};
use serde::{Deserialize, Serialize};
use warp::{query, Filter, Rejection, Reply};

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Response {
    // blank
    extra: String,
    // blank
    #[serde(rename = "ID")]
    id: String,
    // the actual response
    responses: Vec<ResponsesMember>,
    // TODO: what's this?
    r#type: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ResponsesMember {
    // Multiaddrs
    addrs: Vec<String>,
    // PeerId
    #[serde(rename = "ID")]
    id: String,
}

#[derive(Debug, Deserialize)]
pub struct FindPeerQuery {
    arg: String,
    // FIXME: doesn't seem to be used at the moment
    verbose: Option<bool>,
    timeout: Option<StringSerialized<humantime::Duration>>,
}

async fn find_peer_query<T: IpfsTypes>(
    ipfs: Ipfs<T>,
    query: FindPeerQuery,
) -> Result<impl Reply, Rejection> {
    let FindPeerQuery {
        arg,
        verbose: _,
        timeout,
    } = query;
    let peer_id = arg.parse::<PeerId>().map_err(StringError::from)?;
    let addrs = ipfs
        .find_peer(peer_id.clone())
        .maybe_timeout(timeout.map(StringSerialized::into_inner))
        .await
        .map_err(StringError::from)?
        .map_err(StringError::from)?
        .into_iter()
        .map(|addr| addr.to_string())
        .collect();
    let id = peer_id.to_string();

    let response = Response {
        extra: Default::default(),
        id: Default::default(),
        responses: vec![ResponsesMember { addrs, id }],
        r#type: 2,
    };

    Ok(warp::reply::json(&response))
}

pub fn find_peer<T: IpfsTypes>(
    ipfs: &Ipfs<T>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    with_ipfs(ipfs)
        .and(query::<FindPeerQuery>())
        .and_then(find_peer_query)
}

#[derive(Debug, Deserialize)]
pub struct FindProvidersQuery {
    arg: String,
    // FIXME: in go-ipfs this returns a lot of logs
    verbose: Option<bool>,
    #[serde(rename = "num-providers")]
    num_providers: Option<usize>,
    timeout: Option<StringSerialized<humantime::Duration>>,
}

async fn find_providers_query<T: IpfsTypes>(
    ipfs: Ipfs<T>,
    query: FindProvidersQuery,
) -> Result<impl Reply, Rejection> {
    let FindProvidersQuery {
        arg,
        verbose: _,
        num_providers,
        timeout,
    } = query;
    let providers = ipfs
        .get_providers(arg.into_bytes())
        .maybe_timeout(timeout.map(StringSerialized::into_inner))
        .await
        .map_err(StringError::from)?
        .map_err(StringError::from)?
        .into_iter()
        .take(if let Some(n) = num_providers { n } else { 20 })
        .map(|peer_id| ResponsesMember {
            addrs: vec![],
            id: peer_id.to_string(),
        })
        .collect();

    // FIXME: go-ipfs returns just a list of PeerIds
    let response = Response {
        extra: Default::default(),
        id: Default::default(),
        responses: providers,
        r#type: 2,
    };

    Ok(warp::reply::json(&response))
}

pub fn find_providers<T: IpfsTypes>(
    ipfs: &Ipfs<T>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    with_ipfs(ipfs)
        .and(query::<FindProvidersQuery>())
        .and_then(find_providers_query)
}

#[derive(Debug, Deserialize)]
pub struct ProvideQuery {
    arg: String,
    // FIXME: in go-ipfs this returns a lot of logs
    verbose: Option<bool>,
    timeout: Option<StringSerialized<humantime::Duration>>,
}

async fn provide_query<T: IpfsTypes>(
    ipfs: Ipfs<T>,
    query: ProvideQuery,
) -> Result<impl Reply, Rejection> {
    let ProvideQuery {
        arg,
        verbose: _,
        timeout,
    } = query;
    let key = arg.into_bytes();
    ipfs.provide(key)
        .maybe_timeout(timeout.map(StringSerialized::into_inner))
        .await
        .map_err(StringError::from)?
        .map_err(StringError::from)?;

    // FIXME: go-ipfs returns nothing on success
    let response = Response {
        extra: Default::default(),
        id: Default::default(),
        responses: vec![],
        r#type: 2,
    };

    Ok(warp::reply::json(&response))
}

pub fn provide<T: IpfsTypes>(
    ipfs: &Ipfs<T>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    with_ipfs(ipfs)
        .and(query::<ProvideQuery>())
        .and_then(provide_query)
}