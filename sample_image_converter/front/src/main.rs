use lambda_http::{run, service_fn, Error, IntoResponse, Request, Response};
use tera::{Context, Tera};
use std::env;

static SIMPLE_TEMPLATE: &'static str = "<!doctype html>
<html>
  <head>
    <meta charset='utf-8'>
    <meta http-equiv='X-UA-Compatible' content='IE=edge,chrome=1'>
    <meta name='viewport' content='width=device-width, initial-scale=1.0, maximum-scale=1.0'>
    <title>Sample Image Converter</title>
    <script src='https://code.jquery.com/jquery-3.4.1.min.js' integrity='sha256-CSXorXvZcTkaix6Yvo6HppcZGetbYMGWSFlBw8HfCJo=' crossorigin='anonymous'></script>
    <script src='https://cdnjs.cloudflare.com/ajax/libs/semantic-ui/2.4.1/semantic.min.js'></script>
    <link rel='stylesheet' href='https://cdnjs.cloudflare.com/ajax/libs/semantic-ui/2.4.1/semantic.min.css'>
    <style type='text/css'>
body {
  background-color: #EEE;
}
body > .grid {
  height: 100%;
}
body > h1 {
  margin-top: 60px !important;
  margin-bottom: 0 !important;
}
.image {
  margin-top: -100px;
}
.column {
  max-width: 100%;
}
#preview {
  max-width: 100%;
}
.ui.segment > a > img {
  max-width: 100%;
}
    </style>
    <script type='text/javascript'>
var OpenModal = function() {
  $('.large.modal').modal('show');
}
var CloseModal = function() {
  $('.large.modal').modal('hide');
}
var toBase64 = function(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.readAsDataURL(file);
    reader.onload = () => resolve(reader.result);
    reader.onerror = error => reject(error);
  });
}
var OnConverted = function() {
  return function(v) {
    App.imgdata = v;
    $('#preview').attr('src', v);
  }
}
var UploadImage = function(elm) {
  if (!!App.imgdata) {
    $(elm).addClass('disabled');
    PutImage();
  }
}
var PutImage = function() {
  const file = $('#image').prop('files')[0];
  App.extension = GetExtension(file.name);
  const data = {filename: file.name, filedata: App.imgdata};
  Request(data, App.init_url, (res)=>{
    App.sid = res.message;
    App.progress = 'convert_jpg';
    CheckProgress();
    $('#info').removeClass('hidden').addClass('visible');
    ScrollBottom();
    setTimeout(function() {
      CheckStatus();
    }, 3000);
  }, (e)=>{
    console.log(e.responseJSON.message);
  });
}
var ChangeImage = function() {
  const file = $('#image').prop('files')[0];
  toBase64(file).then(OnConverted());
}
var CheckStatus = function() {
  const data = {id: App.sid};
  Request(data, App.check_url, (res)=>{
    if (res.message == 'RUNNING') {
      setTimeout(function() {
        CheckStatus();
      }, 30000);
    } else if (res.message == 'SUCCEEDED') {
      console.log(res.message);
    } else {
      console.log(res.message);
      $('#warning').text(res.message).removeClass('hidden').addClass('visible');
      ScrollBottom();
    }
  }, (e)=>{
    console.log(e.responseJSON.message);
  });
}

var Request = function(data, url, callback, onerror) {
  $.ajax({
    type:          'POST',
    dataType:      'json',
    contentType:   'application/json',
    scriptCharset: 'utf-8',
    data:          JSON.stringify(data),
    url:           url
  })
  .done(function(res) {
    callback(res);
  })
  .fail(function(e) {
    onerror(e);
  });
};

var CheckProgress = function() {
  if (!App.sid) {
    $('#warning').text('ID is Empty').removeClass('hidden').addClass('visible');
    return false;
  }
  var url = App.bucket + App.sid.substr(0, 4) + '-' + App.sid.substr(4, 2) + '-' + App.sid.substr(6, 2) + '-' + App.sid.substr(8, 2) + '-' + App.sid.substr(10, 2) + '/' + App.sid;
  switch (App.progress){
  case 'convert_jpg':
    url += '_convert.jpg'
    break;
  case 'convert_png':
    url += '_convert.png'
    break;
  case 'convert_webp':
    url += '_convert.webp'
    break;
  case 'convert_ico':
    url += '_convert.ico'
    break;
  case 'icon_200':
    url += '_icon_200.png'
    break;
  case 'icon_300':
    url += '_icon_300.png'
    break;
  case 'thumbnail_960_540':
    url += '_thumbnail_960_540.' + App.extension
    break;
  case 'thumbnail_1440_810':
    url += '_thumbnail_1440_810.' + App.extension
    break;
  case 'thumbnail_480_270':
    url += '_thumbnail_480_270.' + App.extension
    break;
  }
  CheckExist(url, (res)=>{
      switch (App.progress){
      case 'convert_jpg':
        App.progress = 'convert_png';
        $('#img_convert_jpg_link').removeClass('active').removeClass('loader').attr('href', url);
        $('#img_convert_jpg').attr('src', url);
        ScrollBottom();
        CheckProgress();
        break;
      case 'convert_png':
        App.progress = 'convert_webp';
        $('#img_convert_png_link').removeClass('active').removeClass('loader').attr('href', url);
        $('#img_convert_png').attr('src', url);
        ScrollBottom();
        CheckProgress();
        break;
      case 'convert_webp':
        App.progress = 'convert_ico';
        $('#img_convert_webp_link').removeClass('active').removeClass('loader').attr('href', url);
        $('#img_convert_webp').attr('src', url);
        ScrollBottom();
        CheckProgress();
        break;
      case 'convert_ico':
        App.progress = 'icon_200';
        $('#img_convert_ico_link').removeClass('active').removeClass('loader').attr('href', url);
        $('#img_convert_ico').attr('src', url);
        ScrollBottom();
        CheckProgress();
        break;
      case 'icon_200':
        App.progress = 'icon_300';
        $('#img_icon_200_link').removeClass('active').removeClass('loader').attr('href', url);
        $('#img_icon_200').attr('src', url);
        ScrollBottom();
        CheckProgress();
        break;
      case 'icon_300':
        App.progress = 'thumbnail_960_540';
        $('#img_icon_300_link').removeClass('active').removeClass('loader').attr('href', url);
        $('#img_icon_300').attr('src', url);
        ScrollBottom();
        CheckProgress();
        break;
      case 'thumbnail_960_540':
        App.progress = 'thumbnail_1440_810';
        $('#img_thumbnail_960_540_link').removeClass('active').removeClass('loader').attr('href', url);
        $('#img_thumbnail_960_540').attr('src', url);
        ScrollBottom();
        CheckProgress();
        break;
      case 'thumbnail_1440_810':
        App.progress = 'thumbnail_480_270';
        $('#img_thumbnail_1440_810_link').removeClass('active').removeClass('loader').attr('href', url);
        $('#img_thumbnail_1440_810').attr('src', url);
        ScrollBottom();
        CheckProgress();
        break;
      case 'thumbnail_480_270':
        App.progress = 'finish';
        $('#img_thumbnail_480_270_link').removeClass('active').removeClass('loader').attr('href', url);
        $('#img_thumbnail_480_270').attr('src', url);
        ScrollBottom();
        break;
      }
  }, (e)=>{
    setTimeout(function() {
      CheckProgress();
    }, 2000);
  });
};

var CheckExist = function(url, callback, onerror) {
  $.ajax({
    type: 'HEAD',
    url:  url
  })
  .done(function(res) {
    callback(res);
  })
  .fail(function(e) {
    onerror(e);
  });
};

var ScrollBottom = function() {
  var bottom = document.documentElement.scrollHeight - document.documentElement.clientHeight;
  window.scroll(0, bottom);
}

var GetExtension = function(str) {
  var re = /(?:\\.([^.]+))?$/;
  extension = re.exec(str)[1];
  if (extension == 'jpeg') {
    extension = 'jpg';
  }
  return extension;
}

var App = { sid: '', progress: '', extension: '', imgdata: null, check_url: location.origin + '{{ api_check_path | safe }}', init_url: location.origin + '{{ api_init_path | safe }}', bucket: '{{ bucket | safe }}' };
    </script>
  </head>
  <body>
    <h1 class='ui center aligned header'>Step Functions</h1>
    <h2 class='ui center aligned header'>Image Converter</h2>
    <div class='main ui container'>
      <form class='ui segment' method='POST'>
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
            <div class='field'>
              <div id='submit' class='ui green button' onclick='UploadImage(this);'>Send</div>
            </div>
          </div>
        </div>
      </form>
    </div>
    <div id='warning' class='ui hidden warning message'></div>
    <div id='info' class='ui hidden info message'>
      <i class='close icon'></i>
      <div class='header'>
        Result
      </div>
      <div class='ui unstackable items'>
        <div class='ui three column grid'>
          <div class='column'>
            <div class='ui segment'>
              <a id='img_convert_jpg_link' class='ui active centered inline loader' target='_blank' href=''>
                <img id='img_convert_jpg' src=''>
              </a>
            </div>
          </div>
          <div class='column'>
            <div class='ui segment'>
              <a id='img_convert_png_link' class='ui active centered inline loader' target='_blank' href=''>
                <img id='img_convert_png' src=''>
              </a>
            </div>
          </div>
          <div class='column'>
            <div class='ui segment'>
              <a id='img_convert_webp_link' class='ui active centered inline loader' target='_blank' href=''>
                <img id='img_convert_webp' src=''>
              </a>
            </div>
          </div>
          <div class='column'>
            <div class='ui segment'>
              <a id='img_convert_ico_link' class='ui active centered inline loader' target='_blank' href=''>
                <img id='img_convert_ico' src=''>
              </a>
            </div>
          </div>
          <div class='column'>
            <div class='ui segment'>
              <a id='img_icon_200_link' class='ui active centered inline loader' target='_blank' href=''>
                <img id='img_icon_200' src=''>
              </a>
            </div>
          </div>
          <div class='column'>
            <div class='ui segment'>
              <a id='img_icon_300_link' class='ui active centered inline loader' target='_blank' href=''>
                <img id='img_icon_300' src=''>
              </a>
            </div>
          </div>
          <div class='column'>
            <div class='ui segment'>
              <a id='img_thumbnail_960_540_link' class='ui active centered inline loader' target='_blank' href=''>
                <img id='img_thumbnail_960_540' src=''>
              </a>
            </div>
          </div>
          <div class='column'>
            <div class='ui segment'>
              <a id='img_thumbnail_1440_810_link' class='ui active centered inline loader' target='_blank' href=''>
                <img id='img_thumbnail_1440_810' src=''>
              </a>
            </div>
          </div>
          <div class='column'>
            <div class='ui segment'>
              <a id='img_thumbnail_480_270_link' class='ui active centered inline loader' target='_blank' href=''>
                <img id='img_thumbnail_480_270' src=''>
              </a>
            </div>
          </div>
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
    let env_api_check_path = env::var("API_CHECK_PATH").unwrap();
    let env_api_init_path = env::var("API_INIT_PATH").unwrap();
    let env_bucket_name = env::var("BUCKET_NAME").unwrap();
    let env_region = env::var("REGION").unwrap();

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![("sample.html", SIMPLE_TEMPLATE)]).unwrap();
    let mut ctx = Context::new();
    ctx.insert("api_check_path", &env_api_check_path);
    ctx.insert("api_init_path", &env_api_init_path);
    ctx.insert("bucket", &format!("https://{}.s3-{}.amazonaws.com/", env_bucket_name.to_string(), env_region.to_string()));

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
