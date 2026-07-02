# OpenMontage Notes

## Вердикт

По состоянию на **1 июля 2026**: **да, можно**, но не в смысле «только custom agent + skills, и всё заработает».
`OpenMontage` уже сделан как **instruction-driven** система: агент читает pipeline manifests и stage skills, а Python-слой даёт tools/checkpoints/persistence. В README проект прямо пишет, что работает с `Copilot` и `Codex`, и в репо уже есть `AGENT_GUIDE.md`, `CODEX.md`, `COPILOT.md`.
Источники: [README](https://github.com/calesthio/OpenMontage), [AGENT_GUIDE](https://github.com/calesthio/OpenMontage/blob/main/AGENT_GUIDE.md), [PROVIDERS](https://github.com/calesthio/OpenMontage/blob/main/docs/PROVIDERS.md)

## Что это значит на практике

- **Для Codex** это почти нативный сценарий. У Codex есть `AGENTS.md`, repo/user/system `skills`, custom agents, plugins и MCP. По официальной странице, Codex включён в ChatGPT `Free/Go/Plus/Pro/Business/Edu/Enterprise`; **API key** у Codex это уже отдельный pay-per-token режим.
  Источники: [Codex Pricing](https://developers.openai.com/codex/pricing), [Codex Skills](https://developers.openai.com/codex/skills), [Codex Subagents](https://developers.openai.com/codex/subagents)
- **Для GitHub Copilot** тоже реально. У платных планов есть `cloud agent`, `custom agents`, `agent skills`, `repository custom instructions` и `MCP`. Но Copilot считает usage в **AI credits**; если лимит кончится, дальше идёт доп. биллинг.
  Источники: [Plans](https://docs.github.com/en/copilot/get-started/plans), [Custom agents](https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/customize-cloud-agent/create-custom-agents), [Agent skills](https://docs.github.com/en/copilot/concepts/agents/about-agent-skills), [MCP](https://docs.github.com/en/copilot/concepts/context/mcp), [Billing](https://docs.github.com/en/copilot/concepts/billing/usage-based-billing-for-individuals)
- Важная тонкость: GitHub Docs пишет, что **OpenAI Codex coding agent** доступен на всех paid Copilot plans, но вход в **OpenAI Codex VS Code extension** через Copilot доступен только для `Pro+` и `Max`.
  Источник: [OpenAI Codex in Copilot](https://docs.github.com/en/copilot/concepts/agents/openai-codex)

## Где всё равно будут деньги

Подписка `Codex/Copilot` покрывает **мозг агента**, но не заменяет **медиа-провайдеров**.
У `OpenMontage` есть бесплатный/локальный путь: `Piper TTS`, open footage (`Archive.org/NASA/Wikimedia`), free stock, `Remotion/HyperFrames`, `FFmpeg`. Но `Runway`, `fal`, `OpenAI image/TTS`, `Google Imagen`, `ElevenLabs`, `HeyGen` и т.д. останутся отдельными вызовами и отдельной стоимостью.

## Мой вывод

Самый разумный путь: **целиться сначала в Codex**, не переписывать OpenMontage в «чистые skills», а оставить его существующий Python tool-layer и дать агенту доступ к нему через терминал/MCP/plugin.
То есть: **да, оркестрацию можно посадить на prepaid Codex/Copilot; нет, это не уберёт стоимость самих video/image/voice API; и skills без tool-layer недостаточно.**

EXPANDABLE: детали минимального порта в Codex, детали порта в Copilot, cheapest-only конфигурация OpenMontage.

------

Если говорить **пессимистично, но без абсурда**, я бы закладывал **$30-70 за 1 минуту** после перехода на мозг Codex, **если видео реально собирается через платные media API**, а не только из бесплатного стока и локальных инструментов.

Почему так:

- **TTS почти ничего не стоит**: у OpenAI в таблице OpenMontage это `$12-30 / 1M chars`, то есть минута озвучки обычно даёт **сенты**, не доллары.
- **Картинки тоже не главный драйвер**: DALL-E 3 в их же гайде стоит примерно `$0.04-0.12` за изображение, то есть даже `10-20` кадров это обычно **$1-3**.
- **Деньги съедает именно video generation**:
  - `Runway Gen-4 Turbo` около `~$0.05/сек`
  - `Runway Gen-4 Aleph` около `~$0.15/сек`
  - `Kling` около `~$0.07-0.10/сек`
  - `Veo` около `~$0.40-0.50/сек`

Пессимистичный расчёт я бы делал не из **60 секунд финала**, а из **120-180 секунд реально нагенерённого материала**, потому что будут рероллы:

- **бюджетный motion-path** (`Kling`, `Runway Turbo`, `MiniMax`): **$10-25**
- **нормальный пессимистичный прод-бюджет** с рероллами, музыкой, вставками: **$30-70**
- **дорогой path** (`Veo`, `Aleph`, premium gateway`) : **$50-100+**

Коротко:
**если не лезть в Veo и не делать много перегенераций, держи в голове `$20-40`; если хочешь именно пессимистичную оценку, ставь `$30-70`; если разрешаешь premium video models и 2-3 реролла на сцену, легко улетишь в `$80+`.**

Переход на **Codex как мозг** эту смету почти не меняет: если ты сидишь на prepaid плане, его маржинальная цена на один ролик обычно теряется на фоне media API. Если же пойдёшь через API-key Codex, я бы **сверху** держал ещё буфер на оркестрацию, но это уже обычно не главный cost driver.

Источники:

- [OpenMontage README](https://github.com/calesthio/OpenMontage)
- [OpenMontage PROVIDERS.md](https://github.com/calesthio/OpenMontage/blob/main/docs/PROVIDERS.md)
- [Codex Pricing](https://developers.openai.com/codex/pricing)

EXPANDABLE: формула сметы для 1 минуты, дешёвый стек без Veo, таблица по провайдерам.

------

Да: **`80-100 USD` для 1 минуты это не обычный сценарий**. Это только **верхний край**, если жечь **дорогую video model** и делать много рероллов.

Формула, откуда это берётся:

- допустим, финал = `10 сцен по 6 сек`
- и в среднем ты делаешь `3 генерации на сцену`, чтобы выбрать удачную
- тогда оплачиваемый объём = `10 x 6 x 3 = 180 сек generated video`

Дальше по провайдеру:

- **Veo 3.1 / very expensive tier** около `~$0.50/сек`
  - `180 x 0.50 = $90`
  - плюс картинки, TTS, музыка -> уже около `~$92-100`
- **Runway Gen-4 Aleph** около `~$0.15/сек`
  - `180 x 0.15 = $27`
- **Kling 3.0** около `~$0.10/сек`
  - `180 x 0.10 = $18`
- **Runway Gen-4 Turbo** около `~$0.05/сек`
  - `180 x 0.05 = $9`

То есть:

- **для Kling / Runway Turbo** мой прошлый `80-100` был слишком пессимистичен
- **для Veo-class** он реален
- **для нормального paid стека** честнее говорить:
  - `~$10-25` при умеренных рероллах
  - `~$25-40` если сцен много и ты перебираешь аккуратнее
  - `~$80-100+` только если берёшь **Veo / самый дорогой video tier**

Итого: если твой план не включает `Veo`-уровень цен, то я бы **пересмотрел пессимистичную оценку вниз до `~$15-35` за минуту**.

Источники:

- [OpenMontage PROVIDERS.md](https://github.com/calesthio/OpenMontage/blob/main/docs/PROVIDERS.md)
- [OpenMontage README](https://github.com/calesthio/OpenMontage)

EXPANDABLE: точная смета для `Kling`, `Runway`, `Veo` по сценам.
