import type { Segment } from "../types";

export function mockSegmentsForFile(fileId: string): Segment[] {
  const base = [
    {
      startMs: 0,
      endMs: 3120,
      en: "Hello everyone, welcome to As It Sounds.",
      han: "헬로 에브리원, 웰컴 투 애즈 잇 사운즈.",
      ko: "안녕하세요 여러분, As It Sounds에 오신 것을 환영합니다.",
    },
    {
      startMs: 3120,
      endMs: 6480,
      en: "This app converts English audio into Korean phonetic spelling.",
      han: "디스 앱 컨버츠 잉글리시 오디오 인투 코리안 포네틱 스펠링.",
      ko: "이 앱은 영어 오디오를 한국어 발음 표기로 변환합니다.",
    },
    {
      startMs: 6480,
      endMs: 10240,
      en: "You can also see the translation right below it.",
      han: "유 캔 올소 시 더 트랜슬레이션 라잇 빌로우 잇.",
      ko: "또한 바로 아래에서 번역을 볼 수 있습니다.",
    },
    {
      startMs: 10240,
      endMs: 13880,
      en: "Drag and drop your audio or video files to get started.",
      han: "드래그 앤 드롭 유어 오디오 오어 비디오 파일즈 투 겟 스타티드.",
      ko: "오디오나 영상 파일을 끌어다 놓으면 시작됩니다.",
    },
  ];
  return base.map((b, i) => ({
    id: `${fileId}-seg-${i}`,
    fileId,
    index: i,
    startMs: b.startMs,
    endMs: b.endMs,
    textEn: b.en,
    textHangul: b.han,
    textKo: b.ko,
  }));
}
