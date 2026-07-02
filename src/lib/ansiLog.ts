export type AnsiLogColor =
  | "red"
  | "green"
  | "yellow"
  | "blue"
  | "magenta"
  | "cyan"
  | "white";

export interface AnsiLogSegment {
  text: string;
  color?: AnsiLogColor;
  bold?: boolean;
}

interface AnsiStyle {
  color?: AnsiLogColor;
  bold?: boolean;
}

const sgrPattern = /\x1b\[([0-9;]*)m/g;

export function parseAnsiLogLine(input: string): AnsiLogSegment[] {
  const segments: AnsiLogSegment[] = [];
  const style: AnsiStyle = {};
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = sgrPattern.exec(input)) !== null) {
    pushSegment(segments, input.slice(lastIndex, match.index), style);
    applySgrCodes(style, match[1]);
    lastIndex = match.index + match[0].length;
  }

  pushSegment(segments, input.slice(lastIndex), style);
  return segments.length > 0 ? segments : [{ text: "" }];
}

function pushSegment(
  segments: AnsiLogSegment[],
  text: string,
  style: AnsiStyle,
) {
  if (!text) return;
  const segment: AnsiLogSegment = { text };
  if (style.color) segment.color = style.color;
  if (style.bold) segment.bold = true;
  segments.push(segment);
}

function applySgrCodes(style: AnsiStyle, rawCodes: string) {
  const codes = rawCodes
    ? rawCodes.split(";").map((code) => Number(code || "0"))
    : [0];

  for (const code of codes) {
    if (code === 0) {
      delete style.color;
      delete style.bold;
    } else if (code === 1) {
      style.bold = true;
    } else if (code === 22) {
      delete style.bold;
    } else if (code === 39) {
      delete style.color;
    } else {
      const color = colorForSgrCode(code);
      if (color) style.color = color;
    }
  }
}

function colorForSgrCode(code: number): AnsiLogColor | undefined {
  switch (code) {
    case 31:
    case 91:
      return "red";
    case 32:
    case 92:
      return "green";
    case 33:
    case 93:
      return "yellow";
    case 34:
    case 94:
      return "blue";
    case 35:
    case 95:
      return "magenta";
    case 36:
    case 96:
      return "cyan";
    case 37:
    case 97:
      return "white";
    default:
      return undefined;
  }
}
