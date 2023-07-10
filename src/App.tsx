import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

function App() {
  return <Main />;
}

const Main = () => {
  const totalCells = 25;
  const [inputs, setInputs] = useState<string[]>(
    new Array(totalCells).fill("")
  );
  const [log, setLog] = useState<string>("");

  const inputsToString = () => {
    const input = inputs.map((x) => x.split("").sort().reverse().join(""));
    let out = "";
    for (let i = 0; i < 5; i++) {
      for (let j = 0; j < 5; j++) {
        out += input[i * 5 + j];
      }
      out += "\n";
    }
    return out;
  };

  const reset = () => {
    setInputs(new Array(totalCells).fill(""));
  };

  const handleInputChange = (e: any, index: number) => {
    const newInputs = [...inputs];
    newInputs[index] = e.target.value;
    setInputs(newInputs);
  };

  const handleKeyDown = (e: any, index: number) => {
    if (e.key === "Backspace") {
      if (e.target.value === "" && index > 0) {
        document.getElementById(`cell-${index - 1}`)?.focus();
      } else {
        const newInputs = [...inputs];
        newInputs[index] = newInputs[index].slice(0, -1);
        setInputs(newInputs);
      }
    }
  };

  useEffect(() => {
    const nextEmptyCellIndex = inputs.findIndex((input) => !input);
    if (nextEmptyCellIndex !== -1) {
      if (nextEmptyCellIndex !== 0) {
        const currentCell = inputs[nextEmptyCellIndex - 1];
        if (
          currentCell.length > 0 &&
          currentCell.split("").at(-1)?.match(/[0-9]/)
        ) {
          return;
        }
      }
      document.getElementById(`cell-${nextEmptyCellIndex}`)?.focus();
    }
  }, [inputs]);

  return (
    <>
      <div style={{ display: "flex", flexDirection: "row" }}>
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(5, 1fr)",
            height: "200px",
          }}
        >
          {new Array(totalCells).fill(null).map((_, i) => (
            <input
              id={`cell-${i}`}
              key={i}
              value={inputs[i]}
              onChange={(e) => handleInputChange(e, i)}
              onKeyDown={(e) => handleKeyDown(e, i)}
              style={{
                width: "30px",
                height: "20px",
                margin: "5px",
                fontFamily:
                  "Operator Mono,Source Code Pro,Menlo,Monaco,Consolas,Courier New,monospace",
              }}
              maxLength={3}
            />
          ))}
          <button
            onClick={async () => {
              if (inputs.some((x) => !x.match(/[a-z]/))) {
                setLog("Invalid input!!!");
                setTimeout(() => {
                  setLog("");
                }, 1000);
                return;
              }
              const log = (await invoke("exec", {
                input: inputsToString(),
              })) as string;
              console.log(log);
              setLog(log);
            }}
          >
            Solve
          </button>
          <button onClick={reset}>Reset</button>
          <div></div>
          <div>
            0 : 2x <br />
            1 : diamond <br />
            2 : DL <br />
            3 : TL <br />
          </div>
        </div>
        <div style={{ marginLeft: "10px" }}>
          <code>
            {log.split("\n").map((x, i) => (
              <div key={i}>{x}</div>
            ))}
          </code>
        </div>
        <br />
      </div>
    </>
  );
};

export default App;
