# ypbank-tools

Библиотека и два консольных приложения для чтения, конвертации и сравнения
финансовых операций. Поддерживаются форматы `csv`, `text` и `bin`.

## Сборка и проверки

```bash
cargo build
cargo test
cargo fmt --check
```

## Форматы

### csv

Первая строка — заголовок.

```csv
date,category,kind,amount
2026-04-01,salary,income,120000
2026-04-02,rent,expense,40000
```

### text

Одна строка — одна операция.

```text
2026-04-01;salary;income;120000
2026-04-02;rent;expense;40000
```

### bin

Детерминированный бинарный формат, все числа little-endian.

| Поле | Размер | Описание |
|---|---|---|
| `magic` | 3 байта | `YPB` (`59 50 42`) |
| `version` | 1 байт | версия формата, сейчас `1` |
| `count` | 4 байта | количество записей, `u32` |
| `date_len` + `date` | 2 + N байт | длина `u16` и UTF-8 дата `YYYY-MM-DD` |
| `category_len` + `category` | 2 + N байт | длина `u16` и UTF-8 категория |
| `kind` | 1 байт | `0` — income, `1` — expense |
| `amount` | 8 байт | сумма, `u64` |

Запись повторяется `count` раз. Разделителей и выравнивания нет.
Максимальная длина строки — 1024 байта.

## converter

Читает файл или stdin, пишет результат в stdout.

```bash
cargo run -- --input-format csv --output-format bin --input data.csv > data.bin
cargo run -- --input-format bin --output-format csv --input data.bin
cat data.csv | cargo run -- --input-format csv --output-format text
```

Без `--input` данные читаются из stdin. Ошибки печатаются в stderr.

## comparer

Парсит два файла и сравнивает списки транзакций.

```bash
cargo run --bin comparer -- \
  --left data.csv --left-format csv \
  --right data.text --right-format text
```

Вывод при совпадении:

```text
files are equal
```

Вывод при различии:

```text
transactions differ at position 3: amount: 6500 != 7000
```

Сравнение идёт после парсинга: важен порядок записей, количество должно
совпадать, позиции нумеруются с 1.

## Тестовые данные

`data.csv` — пример на шесть операций для ручных проверок.
