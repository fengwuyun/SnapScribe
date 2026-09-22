# Speaker diarization runtime and models

SnapScribe bundles the following files for local speaker diarization. All URLs are official upstream release assets.

| File | Source | SHA-256 |
| --- | --- | --- |
| `bin/sherpa-onnx-offline-speaker-diarization.exe` | `sherpa-onnx-v1.13.8-win-x64-shared-MD-Release-no-tts.tar.bz2` | `968FB3E33E293D36D23329F475BDEF934925B97AC09F897D3BDC97085E506CF3` |
| `bin/onnxruntime.dll` | same Sherpa-ONNX v1.13.8 archive | `422D776AB0E3218260F7F628FCB84606AAA5C21116F720C8619A6DA5E0B2E0F9` |
| `bin/onnxruntime_providers_shared.dll` | same Sherpa-ONNX v1.13.8 archive | `0190137DEE4933261C065C5D030C3568A68C7AA43CAD942B4726A07011738BC2` |
| `models/speaker-segmentation/model.int8.onnx` | `sherpa-onnx-pyannote-segmentation-3-0.tar.bz2` | `D582F4B4C6B48205DE7E0643C57DF0DF5615A3C176189BE3FC461E9D18827B5D` |
| `models/speaker-embedding/3dspeaker.onnx` | `3dspeaker_speech_eres2net_base_sv_zh-cn_3dspeaker_16k.onnx` | `1A331345F04805BADBB495C775A6DDFFCDD1A732567D5EC8B3D5749E3C7A5E4B` |

Sources:

- https://github.com/k2-fsa/sherpa-onnx/releases/tag/v1.13.8
- https://github.com/k2-fsa/sherpa-onnx/releases/tag/speaker-segmentation-models
- https://github.com/k2-fsa/sherpa-onnx/releases/tag/speaker-recongition-models

License references:

- Sherpa-ONNX: Apache-2.0, https://github.com/k2-fsa/sherpa-onnx/blob/v1.13.8/LICENSE
- ONNX Runtime: MIT, https://github.com/microsoft/onnxruntime/blob/main/LICENSE
- Pyannote segmentation model: review upstream model terms at https://huggingface.co/pyannote/segmentation-3.0
- 3D-Speaker model: review upstream project terms at https://github.com/modelscope/3D-Speaker
