use serde::{Deserialize, Serialize};

use worker::*;


static NAMESPACE: &'static str = "webrtc_session";

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    utils::set_panic_hook();
    let router = Router::new();
    router
        .get_async("/", list_sessions)
        .get_async("/:id", get_session)
        .post_async("/:id", create_session)
        .delete_async("/:id", delete_session)
        .options_async("/:id", preflight)
        .get_async("/test", test)
        .run(req, env)
        .await
        .map(|res| with_cors(res))
}

async fn test(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let client = redis::Client::open().unwrap();
    let mut con = client.get_connection().unwrap();
    let _: () = redis::cmd("SET")
        .arg("my_key")
        .arg("42")
        .query(&mut con)
        .unwrap();
    let bar: String = redis::cmd("GET").arg("my_key").query(&mut con).unwrap();
    let res = Response::ok(bar).unwrap();
    return Ok(res);
}

async fn list_sessions(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let store = match ctx.kv(NAMESPACE) {
        Ok(s) => s,
        Err(err) => return Response::error(format!("{:?}", err), 204),
    };

    let list = match store.list().execute().await {
        Ok(l) => l.keys,
        err => return Response::error(format!("server error: {:?}", err), 500),
    };

    let res = Response::from_json(&list).unwrap();
    return Ok(res);
}

async fn get_session(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let store = match ctx.kv(NAMESPACE) {
        Ok(s) => s,
        Err(err) => return Response::error(format!("{:?}", err), 204),
    };

    let session_id: &String = match ctx.param("id") {
        Some(id) => id,
        None => return Response::error(format!("session not found"), 404),
    };

    match store.get(session_id).json::<Session>().await {
        Ok(Some(session)) => Ok(Response::from_json(&session).unwrap()),
        Ok(None) => Response::error(format!("session not found"), 404),
        Err(err) => Response::error(format!("store get: {:?}", err), 500),
    }
}

async fn create_session(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let store = match ctx.kv(NAMESPACE) {
        Ok(s) => s,
        Err(err) => return Response::error(format!("{:?}", err), 204),
    };

    let session_id: &String = match ctx.param("id") {
        Some(id) => id,
        None => return Response::error(format!("session not found"), 404),
    };

    let peer: Peer = match req.json::<Peer>().await {
        Ok(session) => session,
        Err(err) => {
            return Response::error(
                format!("body parse error: {:?} in {:?}", err, req.text().await),
                400,
            )
        }
    };

    match store.get(session_id).json::<Session>().await {
        Ok(Some(mut session)) => {
            session.push(peer);
            let put = store.put(session_id, session);
            if put.is_ok() {
                let exc = put.unwrap().execute().await;
                if exc.is_ok() {
                    let res = Response::ok("success").unwrap();
                    Ok(res)
                } else {
                    Response::error("storage error", 500)
                }
            } else {
                Response::error("storage error", 500)
            }
        }
        Ok(None) => {
            let session: Session = vec![peer];
            let put = store.put(session_id, session);
            if put.is_ok() {
                let exc = put.unwrap().execute().await;
                if exc.is_ok() {
                    let res = Response::ok("success").unwrap();
                    Ok(res)
                } else {
                    Response::error("storage error", 500)
                }
            } else {
                Response::error("storage error", 500)
            }
        }
        Err(err) => return Response::error(format!("server error: {:?}", err), 500),
    }
}

async fn delete_session(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let store = match ctx.kv(NAMESPACE) {
        Ok(s) => s,
        Err(err) => return Response::error(format!("{:?}", err), 204),
    };

    let session_id: &String = match ctx.param("id") {
        Some(id) => id,
        None => return Response::error(format!("session not found"), 404),
    };

    let put = store.delete(session_id).await;
    if put.is_ok() {
        let res = Response::ok("success").unwrap();
        return Ok(res);
    } else {
        return Response::error("storage error", 500);
    }
}

async fn preflight(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Ok(Response::ok("success").unwrap())
}

fn with_cors(res: Response) -> Response {
    let mut headers = Headers::default();
    headers
        .append(&"Access-Control-Allow-Origin".to_string(), &"*".to_string())
        .unwrap();
    headers
        .append(
            &"Access-Control-Allow-Methods".to_string(),
            &"GET, POST, OPTIONS, PUT, DELETE".to_string(),
        )
        .unwrap();
    headers
        .append(&"Content-type".to_string(), &"application/json".to_string())
        .unwrap();
    headers
        .append(&"Access-Control-Max-Age".to_string(), &"86400".to_string())
        .unwrap();
    res.with_headers(headers)
}

