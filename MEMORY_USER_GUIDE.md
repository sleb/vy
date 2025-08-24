# Vy Memory System User Guide 🧠

Welcome to Vy's powerful memory system! Your AI assistant can now remember, search, and manage information across conversations.

## 🚀 Quick Start

Simply chat naturally with Vy and use memory commands like:

- **"Remember that I love hiking on weekends"**
- **"What do I like to do for exercise?"**
- **"Update my weekend activity to cycling instead"**
- **"Forget about my hiking preference"**

## 💬 Natural Language Commands

### Storing Memories
Tell Vy to remember anything important:

```
💬 You: Remember that I'm allergic to peanuts
🤖 Vy: 🧠 Storing memory... ✅

🧠 **Memory Storage Results**

**Status:** ✅ Success
**Message:** Fact stored successfully in vector memory
**Stored Fact:** I'm allergic to peanuts
```

**Other ways to store:**
- "Keep in mind that I prefer morning meetings"
- "Note that my favorite coffee shop is Blue Bottle"
- "Don't forget I'm vegetarian"

### Searching Memories
Ask Vy to recall information:

```
💬 You: What are my dietary restrictions?
🤖 Vy: 🔍 Searching memories... ✅

🔍 **Memory Search Results**
**Query:** dietary restrictions
**Found:** 2 memories

**1. I'm allergic to peanuts** (Score: 0.95)
   *Source: user*

**2. I'm vegetarian** (Score: 0.87)
   *Source: user*
```

**Other ways to search:**
- "What do you know about my work schedule?"
- "Remind me about my preferences"
- "What have I told you about my family?"

### Updating Memories
Change or correct stored information:

```
💬 You: Actually, I'm not vegetarian anymore - I eat chicken now
🤖 Vy: 🔄 Updating memory... ✅

🔄 **Memory Update Results**

**Status:** ✅ Success
**Message:** Successfully updated 1 memory(ies)
**Old Fact:** I'm vegetarian
**New Fact:** I eat chicken now
```

### Removing Memories
Delete unwanted or outdated information:

```
💬 You: Forget about my old job at Microsoft
🤖 Vy: 🗑️ Removing memories... ✅

🗑️ **Memory Removal Results**

**Status:** ✅ Success
**Query:** old job at Microsoft
**Message:** Successfully removed 1 memory(ies)
**Removed:** 1 memory(ies)
```

## 🤖 Automatic Memory Learning

Vy automatically identifies and stores important information from your conversations:

- Personal preferences and interests
- Important dates and events
- Work and project details
- Family and relationship information
- Goals and aspirations
- Health and dietary information

**Example automatic learning:**
```
💬 You: I just got promoted to Senior Developer at Google!
🤖 Vy: Congratulations on your promotion! 🎉

(Vy automatically stores: "User got promoted to Senior Developer at Google")
```

## 🔍 How Memory Search Works

Vy uses **semantic search** - it understands meaning, not just keywords:

- **Query:** "What do I do for work?"
- **Finds:** "User is a software engineer at Google"
- **Also finds:** "User got promoted to Senior Developer"

This means you can ask in different ways and still get relevant results.

## 💡 Memory Tips & Best Practices

### What to Store
✅ **Good for memory:**
- Personal preferences ("I love spicy food")
- Important dates ("My birthday is March 15th")
- Work information ("I work remotely on Tuesdays")
- Goals ("I want to learn Spanish this year")
- Contacts ("My dentist is Dr. Smith at 555-1234")

❌ **Not ideal for memory:**
- Temporary information ("It's raining today")
- Constantly changing data ("Gas costs $3.50")
- Private/sensitive data (passwords, SSNs)

### Memory Commands That Work Well
- **Store:** "Remember that...", "Keep in mind...", "Don't forget..."
- **Search:** "What do you know about...", "Tell me about my...", "Remind me..."
- **Update:** "Actually...", "Change that to...", "I prefer... now"
- **Remove:** "Forget about...", "Delete my...", "Remove the memory about..."

### Getting Better Results
1. **Be specific:** "I prefer dark roast coffee" vs "I like coffee"
2. **Use context:** "Remember my work schedule: Mon-Fri 9-5 EST"
3. **Update when things change:** "Update my address to..."
4. **Regular maintenance:** Occasionally review and clean up old memories

## 🛠️ Technical Details

### Behind the Scenes
- **Vector Database:** Qdrant cloud service for fast semantic search
- **Embeddings:** OpenAI's text-embedding-3-small for understanding meaning
- **Storage:** Memories persist across all conversations and sessions
- **Search:** Sub-second semantic similarity matching

### Privacy & Security
- **Your data:** Stored securely in your personal Qdrant cloud instance
- **Encryption:** All communications encrypted in transit
- **Control:** You can remove any memory at any time
- **Local:** No memory data shared between users

## 🚨 Troubleshooting

### Common Issues

**Memory not storing?**
- Check your OpenAI API key: `vy config list`
- Ensure internet connection for cloud storage
- Try more explicit commands: "Remember exactly: [fact]"

**Search not finding memories?**
- Try different phrasings of your query
- Use broader terms: "work" instead of "specific project name"
- Check if the memory was actually stored: search for keywords

**Updates not working?**
- Be specific about what to change
- Include both old and new information in your request
- Try removing and re-adding if updates fail

### Getting Help
- Use `vy config list` to check configuration
- Look for error messages in the output
- Try basic operations first: store, then search
