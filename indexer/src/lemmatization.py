# Install necessary libraries:
# pip install fastapi uvicorn spacy
# python -m spacy download en_core_web_sm

from fastapi import FastAPI
from pydantic import BaseModel
import spacy

# Initialize spaCy model
nlp = spacy.load("en_core_web_sm")

# Create FastAPI app
app = FastAPI()

# Define input schema
class Tokens(BaseModel):
    tokens: list[str]

@app.get("/favicon.ico")
def favicon():
    return {"message": "No favicon available"}

@app.get("/")
def read_root():
    return {"message": "Lemmatization API is running!"}

# Define lemmatization endpoint
@app.post("/lemmatize")
def lemmatize(tokens: Tokens):
    if not tokens.tokens:
        return {"lemmatized_tokens": []}  # Return empty if no tokens are provided

    doc = nlp(" ".join(tokens.tokens))
    lemmatized_tokens = [token.lemma_ for token in doc if not token.is_punct and not token.is_space]
    return {"lemmatized_tokens": lemmatized_tokens}

