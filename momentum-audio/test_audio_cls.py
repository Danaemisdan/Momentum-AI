from transformers import pipeline
print("Loading audio classifier...")
try:
    classifier = pipeline("audio-classification", model="MIT/ast-finetuned-audioset-10-10-0.4593")
    print("Loaded AST.")
except Exception as e:
    print("Failed AST:", e)
