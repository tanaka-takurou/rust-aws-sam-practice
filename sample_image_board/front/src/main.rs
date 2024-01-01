use aws_config::meta::region::RegionProviderChain;
use lambda_http::{run, service_fn, Error, IntoResponse, Request, Response};
use aws_config::Region;
use aws_sdk_dynamodb::Client;
use tera::{Context, Tera};
use std::env;

static SIMPLE_TEMPLATE: &'static str = "<!doctype html>
<html>
  <head>
    <meta charset='utf-8'>
    <meta http-equiv='X-UA-Compatible' content='IE=edge,chrome=1'>
    <meta name='viewport' content='width=device-width, initial-scale=1.0, maximum-scale=1.0'>
    <title>Sample Image Board</title>
    <script src='https://code.jquery.com/jquery-3.4.1.min.js' integrity='sha256-CSXorXvZcTkaix6Yvo6HppcZGetbYMGWSFlBw8HfCJo=' crossorigin='anonymous'></script>
    <script src='https://cdnjs.cloudflare.com/ajax/libs/semantic-ui/2.4.1/semantic.min.js'></script>
    <link rel='stylesheet' href='https://cdnjs.cloudflare.com/ajax/libs/semantic-ui/2.4.1/semantic.min.css'>
    <style type='text/css'>
.last.container {
  margin-bottom: 300px !important;
}
h1.ui.center.header {
  margin-top: 3em;
}
h2.ui.center.header {
  margin: 4em 0em 2em;
}
h3.ui.center.header {
  margin-top: 2em;
  padding: 2em 0em;
}
.ui.segment > a > img {
  width: 100%;
}
.ui.footer.segment {
  margin: 5em 0em 0em;
  padding: 5em 0em;
}
.ui.container.vmargin {
  margin: 5rem 0em calc(2rem - 0.14285714em );
}
#preview {
  max-width: 60%;
  max-height: 60%;
}
    </style>
    <script type='text/javascript'>
function OpenModal() {
  $('.large.modal').modal('show');
}
function CloseModal() {
  $('.large.modal').modal('hide');
}
function parseJson (data) {
  var res = {};
  for (i = 0; i < data.length; i++) {
    res[data[i].name] = data[i].value;
  }
  return res;
}
function toBase64 (file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.readAsDataURL(file);
    reader.onload = () => resolve(reader.result);
    reader.onerror = error => reject(error);
  });
}
function onConverted () {
  return function(v) {
    App.imgdata = v;
    $('#preview').attr('src', v);
  }
}
function UploadImage(elm) {
  if (!!App.imgdata) {
    $(elm).addClass('disabled');
    putImage();
  }
}
function putImage() {
  const file = $('#image').prop('files')[0];
  const data = {filename: file.name, filedata: App.imgdata};
  $.ajax({
    type:          'POST',
    dataType:      'json',
    contentType:   'application/json',
    scriptCharset: 'utf-8',
    data:          JSON.stringify(data),
    url:           App.url
  })
  .fail(function(e) {
    console.log(e);
  })
  .always(function() {
    window.setTimeout(() => location.reload(true), 1000);
  });
}
function ChangeImage () {
  const file = $('#image').prop('files')[0];
  toBase64(file).then(onConverted());
}
var App = { imgdata: null, url: location.origin + '{{ api_path | safe }}' };
    </script>
  </head>
  <body>
    <div class='ui fixed inverted menu'>
      <div class='ui container'>
        <a href='#' class='item'>Home</a>
      </div>
    </div>
    <div class='main ui container'>
      <h1 class='ui center aligned header'>Sample Image Board</h1>
      <div class='ui unstackable items'>
        <div class='ui three column grid'>
{% for item in items %}
          <div class='column'>
            <div class='ui segment'>
              <a href='{{ item | safe }}'>
                <img src='{{ item | safe }}'>
              </a>
            </div>
          </div>
{% endfor %}
        </div>
      </div>
      <div class='ui primary button' onclick='OpenModal();'>
        Upload
      </div>
      <div class='ui dimmer modals page transition hidden'>
        <div class='ui large modal transition hidden'>
          <form class='ui large modal' method='POST' style='left: auto !important;'>
            <i class='close icon'></i>
            <div class='header'>
              New Image
            </div>
            <div class='content'>
              <div class='ui form'>
                <div class='field'>
                  <img id='preview' src>
                </div>
                <div class='field'>
                  <label>Image File</label>
                  <div class='ui input'>
                    <input id='image' type='file' name='image' accept='image/*' onchange='ChangeImage();'>
                  </div>
                </div>
              </div>
            </div>
            <input type='hidden' name='action' value='createimg'>
            <div class='actions'>
              <div class='ui button' onclick='CloseModal();'>Cancel</div>
              <div class='ui green button' onclick='UploadImage(this);'>Submit</div>
            </div>
          </form>
        </div>
      </div>
    </div>
    <div class='ui inverted vertical footer segment'>
      <div class='ui center aligned container'>
        <div class='ui horizontal inverted small divided link list'>
          <a class='item' href='#'>Contact</a>
        </div>
      </div>
    </div>
  </body>
</html>";

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-http/examples
async fn function_handler(_event: Request) -> Result<impl IntoResponse, Error> {
    let env_table = env::var("IMG_TABLE_NAME").unwrap();
    let env_region = env::var("REGION").unwrap();
    let env_api_path = env::var("API_PATH").unwrap();
    let region_provider = RegionProviderChain::first_try(Region::new(env_region));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    let res: Result<Vec<_>, _> = client
        .scan()
        .table_name(env_table)
        .into_paginator()
        .items()
        .send()
        .collect()
        .await;
    let items: Vec<_> = res.unwrap_or(vec![]);

    // HashMap into a Vec
    let url_list: Vec<String> = items.iter().map(|x| -> String {
        match x.get("url") {
            Some(url) => (url.as_s().unwrap_or(&String::from(""))).to_string(),
            _ => String::from(""),
        }
    }).collect();

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![("sample.html", SIMPLE_TEMPLATE)]).unwrap();
    let mut ctx = Context::new();
    ctx.insert("items", &url_list);
    ctx.insert("api_path", &env_api_path);

    let html = tera.render("sample.html", &ctx).unwrap();

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(html)
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
