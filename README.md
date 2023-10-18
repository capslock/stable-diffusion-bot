# stable-diffusion-bot

A Telegram bot written in Rust that can be connected to a
[Stable Diffusion web UI](https://github.com/AUTOMATIC1111/stable-diffusion-webui)
backend to generate images.

## Usage

### Install

```shell
cargo install --git https://github.com/capslock/stable-diffusion-bot
```

### Configure

#### Prerequisites

* Create a
  [new telegram bot](https://core.telegram.org/bots/features#creating-a-new-bot),
  or have an existing bot token.
* Get the user ids of the users that you'd like to give access to the bot.
  The bot only responds to users whose id is on the `allowed_users` list.
* Set up the `Stable Diffusion web UI` and enable API access.
  [See the wiki](https://github.com/AUTOMATIC1111/stable-diffusion-webui/wiki/API).

#### Setup

Create a `config.toml` file:

```
api_key = "your_telegram_bot_api_key"
allowed_users = [ list, of, telegram, ids ]
db_path = "./db.sqlite"
sd_api_url = "http://localhost:7860"
```

* `api_key` is optional, and can instead be provided via the environment
  variable `SD_TELEGRAM_API_KEY`.
* `allowed_users` must be supplied.
* `db_path` is optional; user settings will not persist on bot restart if not
  provided.
* `sd_api_url` is required and should be set to the url of a
  `Stable Diffusion web UI` API instance.

### Run

```shell
stable-diffusion-bot
```

Alternatively, if you don't want to store your API key in the config file, you
can pass it through as an environment variable:

```shell
SD_TELEGRAM_API_KEY="your_telegram_bot_api_key" stable-diffusion-bot
```

## Using the bot

* `/start` to get started.
* `/help` will list the available commands.

### `txt2img`

Send the bot a prompt and it will generate an image using the default generation
settings. The reply will also have an inline keyboard, giving you options to:
  * rerun the same prompt
  * save the seed for subsequent generations
  * change settings

### `img2img`

Send the bot an image with a caption and it will generate a new image based on
that image and the prompt.

## Advanced

### Configuration

#### Specifying allowed users

* User IDs can be specified individually, or Chat IDs can be specified to permit
  all users in a group chat from using the bot.
* The option `allow_all_users = true` can be set to instead allow any user to
  access the bot. Note that this means if someone finds your bot, there's no way
  to stop them from using it to generate images.

#### Stable Diffusion Settings

You can set the default `txt2img` and `img2img` parameters via the `config.toml` file.

:construction: **TODO**: Document the available options.

```
[txt2img]
steps = 50
cfg_scale = 10
width = 768
height = 768

[img2img]
width = 768
height = 768
```