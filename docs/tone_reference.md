# Telephony Tone Reference (US)

## Multi-frequency tones

|Type          |Frequency 1|Frequency 2|Frequency 3|Frequency 4|Cadence              |
|--------------|-----------|-----------|-----------|-----------|---------------------|
|Dial tone     |350 Hz     |440 Hz     |--         |--         |Constant             |
|Busy tone     |480 Hz     |620 Hz     |--         |--         |0.5 s ON, 0.5 s OFF  |
|Fast busy tone|480 Hz     |620 Hz     |--         |--         |0.25 s ON, 0.25 s OFF|
|Ringing tone  |440 Hz     |480 Hz     |--         |--         |2 s ON, 4 s OFF      |
|Off-hook tone |1400 Hz    |2060 Hz    |2450 Hz    |2600 Hz    |0.1 s ON, 0.1 s OFF  |

## Special Information Tones (SITs)

### Segment durations

* Short duration = 276 ms
* Long duration = 380 ms

### Segment frequencies

|First|Second|Third|
|-----|------|-----|
|High: 985.2 Hz|High: 1428.5 Hz|--
|Low: 913.8 Hz|Low: 1370.6 Hz|Low: 1776.7 Hz

### SIT encodings

|Name|Durations|Frequencies|Description|
|----|---------|-----------|-----------|
|Intercept|short, short, long|low, low, low|_Number changed or disconnected_

## Sources

* https://en.wikipedia.org/wiki/Precise_tone_plan
* https://en.wikipedia.org/wiki/Off-hook_tone
* https://en.wikipedia.org/wiki/Special_information_tone