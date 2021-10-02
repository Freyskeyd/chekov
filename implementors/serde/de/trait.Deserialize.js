(function() {var implementors = {};
implementors["gift_shop"] = [{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/commands/struct.OpenAccount.html\" title=\"struct gift_shop::commands::OpenAccount\">OpenAccount</a>","synthetic":false,"types":["gift_shop::commands::OpenAccount"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/commands/struct.CreateGiftCard.html\" title=\"struct gift_shop::commands::CreateGiftCard\">CreateGiftCard</a>","synthetic":false,"types":["gift_shop::commands::CreateGiftCard"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/commands/struct.CreateOrder.html\" title=\"struct gift_shop::commands::CreateOrder\">CreateOrder</a>","synthetic":false,"types":["gift_shop::commands::CreateOrder"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/commands/struct.AddGiftCardToOrder.html\" title=\"struct gift_shop::commands::AddGiftCardToOrder\">AddGiftCardToOrder</a>","synthetic":false,"types":["gift_shop::commands::AddGiftCardToOrder"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/commands/struct.ValidateOrder.html\" title=\"struct gift_shop::commands::ValidateOrder\">ValidateOrder</a>","synthetic":false,"types":["gift_shop::commands::ValidateOrder"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/commands/struct.CancelOrder.html\" title=\"struct gift_shop::commands::CancelOrder\">CancelOrder</a>","synthetic":false,"types":["gift_shop::commands::CancelOrder"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/events/account/struct.AccountOpened.html\" title=\"struct gift_shop::events::account::AccountOpened\">AccountOpened</a>","synthetic":false,"types":["gift_shop::events::account::AccountOpened"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"enum\" href=\"gift_shop/events/account/enum.MoneyMovementEvent.html\" title=\"enum gift_shop::events::account::MoneyMovementEvent\">MoneyMovementEvent</a>","synthetic":false,"types":["gift_shop::events::account::MoneyMovementEvent"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/events/gift_card/struct.GiftCardCreated.html\" title=\"struct gift_shop::events::gift_card::GiftCardCreated\">GiftCardCreated</a>","synthetic":false,"types":["gift_shop::events::gift_card::GiftCardCreated"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/events/gift_card/struct.GiftCardUsed.html\" title=\"struct gift_shop::events::gift_card::GiftCardUsed\">GiftCardUsed</a>","synthetic":false,"types":["gift_shop::events::gift_card::GiftCardUsed"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/events/order/struct.OrderCreated.html\" title=\"struct gift_shop::events::order::OrderCreated\">OrderCreated</a>","synthetic":false,"types":["gift_shop::events::order::OrderCreated"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/events/order/struct.GiftCardAdded.html\" title=\"struct gift_shop::events::order::GiftCardAdded\">GiftCardAdded</a>","synthetic":false,"types":["gift_shop::events::order::GiftCardAdded"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/events/order/struct.OrderValidated.html\" title=\"struct gift_shop::events::order::OrderValidated\">OrderValidated</a>","synthetic":false,"types":["gift_shop::events::order::OrderValidated"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/events/order/struct.OrderCanceled.html\" title=\"struct gift_shop::events::order::OrderCanceled\">OrderCanceled</a>","synthetic":false,"types":["gift_shop::events::order::OrderCanceled"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/http/struct.OpenAccountPayload.html\" title=\"struct gift_shop::http::OpenAccountPayload\">OpenAccountPayload</a>","synthetic":false,"types":["gift_shop::http::OpenAccountPayload"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/http/struct.CreateGiftCardPayload.html\" title=\"struct gift_shop::http::CreateGiftCardPayload\">CreateGiftCardPayload</a>","synthetic":false,"types":["gift_shop::http::CreateGiftCardPayload"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/http/struct.AddGiftCardToOrderPayload.html\" title=\"struct gift_shop::http::AddGiftCardToOrderPayload\">AddGiftCardToOrderPayload</a>","synthetic":false,"types":["gift_shop::http::AddGiftCardToOrderPayload"]},{"text":"impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"gift_shop/order/struct.Item.html\" title=\"struct gift_shop::order::Item\">Item</a>","synthetic":false,"types":["gift_shop::order::Item"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()