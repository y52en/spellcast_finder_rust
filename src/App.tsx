import { useState, useEffect, cloneElement, ReactElement, useRef } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import styled from "styled-components";
import "./App.css";

const code_font_family =
  "Operator Mono,Source Code Pro,Menlo,Monaco,Consolas,Courier New,monospace";

function App() {
  return <Main />;
}

const StyledDiv = styled.div`
  display: flex;
  flex-direction: column;

  @media (min-width: 770px) {
    flex-direction: row;
  }
`;

const Main = () => {
  const totalCells = 25;
  const [inputs, setInputs] = useState<string[]>(
    new Array(totalCells).fill("")
  );
  const [log, setLog] = useState<string>("");
  const [focusIndex, setFocusIndex] = useState<number>(-1);

  const inputsToString = () => {
    const input = inputs.map((x) =>
      x
        .split("")
        .sort()
        .reverse()
        .map((x) => x.toLowerCase())
        .join("")
    );
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
    setFocusIndex(index);
  };

  const handleKeyDown = (e: any, index: number) => {
    if (e.key === "Backspace") {
      if (e.target.value === "" && index > 0) {
        document.getElementById(`cell-${index - 1}`)?.focus();
      } else {
        const newInputs = [...inputs];
        // newInputs[index] = newInputs[index].slice(0, -1);
        setInputs(newInputs);
      }
    } else if (e.key === "ArrowLeft") {
      if (index > 0) {
        document.getElementById(`cell-${index - 1}`)?.focus();
      }
      e.preventDefault();
      setFocusIndex(index - 1);
    } else if (e.key === "ArrowRight") {
      if (index < totalCells - 1) {
        document.getElementById(`cell-${index + 1}`)?.focus();
      }
      e.preventDefault();
      setFocusIndex(index + 1);
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
      setFocusIndex(nextEmptyCellIndex);
    }
  }, [inputs]);

  useEffect(() => {
    document.getElementById(`cell-${focusIndex}`)?.focus();
  });

  const showErrorMessage = (text: string) => {
    setLog(text);
    setTimeout(() => {
      setLog("");
    }, 1000);
  };

  return (
    <>
      <StyledDiv>
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(5, 1fr)",
            height: "350px",
            width: "400px",
            marginBottom: "10px",
          }}
        >
          {new Array(totalCells).fill(null).map((_, i) => {
            let elm = (
              <input
                id={`cell-${i}`}
                key={i}
                value={inputs[i]}
                onChange={(e) => handleInputChange(e, i)}
                onKeyDown={(e) => handleKeyDown(e, i)}
                onClick={() => setFocusIndex(i)}
                style={{
                  width: "30px",
                  height: "20px",
                  margin: "5px",
                  fontFamily: code_font_family,
                }}
                maxLength={3}
              />
            );
            const input = inputs[i];
            (
              [
                ["0", "2x", "#F000F0", ["0px", "0px"]],
                ["1", "💎", "#F000F0", ["30px", "65px"]],
                ["2", "DL", "yellow", ["0px", "70px"]],
                ["3", "TL", "red", ["0px", "70px"]],
              ] as [string, string, string, [string, string]][]
            ).forEach(([key, value, color, pos]) => {
              if (input.includes(key)) {
                elm = (
                  <Badge badgeText={value} position={pos} color={color}>
                    {elm}
                  </Badge>
                );
              }
            });
            return elm;
          })}
          <button
            style={{ width: "80px", textAlign: "center", padding: "5px" }}
            onClick={async (self) => {
              if (inputs.some((x) => !x.match(/[a-zA-Z]/))) {
                showErrorMessage("Invalid input!!!");
                return;
              }
              if (!(self.target instanceof HTMLButtonElement)) {
                return;
              }
              self.target.textContent = "Solving...";
              let log = "";
              try {
                log = (await invoke("exec", {
                  input: inputsToString(),
                })) as string;
              } catch (e) {
                showErrorMessage("Error!!!");
                self.target.textContent = "Solve";
                return;
              }
              setLog(log);
              self.target.textContent = "Solve";
            }}
          >
            Solve
          </button>
          <button style={{ textAlign: "center" }} onClick={reset}>
            Reset
          </button>
          <div></div>
          <div>
            0 : 2x <br />
            1 : 💎 <br />
            2 : DL <br />
            3 : TL <br />
          </div>
        </div>
        <div style={{ marginLeft: "10px", width: "350px" }}>
          <code style={{ fontFamily: code_font_family }}>
            {log.split("\n").map((x, i) => (
              <div key={i}>{x}</div>
            ))}
          </code>
        </div>
        <br />
      </StyledDiv>
    </>
  );
};

function Badge({
  children,
  badgeText,
  position,
  color,
}: {
  children: ReactElement;
  badgeText: string;
  position: [string, string];
  color: string;
}) {
  return (
    <div
      style={{
        position: "relative",
        display: "inline-block",
      }}
    >
      {children}
      <span
        style={{
          position: "absolute",
          top: position[0],
          right: position[1],
          backgroundColor: "#111",
          color: "white",
          borderRadius: "50%",
          padding: "1px 2px",
          fontSize: "10px",
          lineHeight: "14px",
          borderColor: color,
          borderWidth: "2px",
          borderStyle: "solid",
        }}
      >
        {badgeText}
      </span>
    </div>
  );
}

export default App;
