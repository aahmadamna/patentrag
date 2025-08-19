import { Card, CardContent } from "@/components/ui/card";

interface Props {
  answer: string;
}

export function AnswerBox({ answer }: Props) {
  if (!answer) return null;

  return (
    <Card>
      <CardContent className="p-4">
        <h2 className="font-semibold mb-2">Answer</h2>
        <p>{answer}</p>
      </CardContent>
    </Card>
  );
}
