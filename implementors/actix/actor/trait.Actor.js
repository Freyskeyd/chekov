(function() {var implementors = {};
implementors["chekov"] = [{"text":"impl&lt;A:&nbsp;<a class=\"trait\" href=\"chekov/trait.Aggregate.html\" title=\"trait chekov::Aggregate\">Aggregate</a>&gt; Actor for <a class=\"struct\" href=\"chekov/prelude/struct.AggregateInstance.html\" title=\"struct chekov::prelude::AggregateInstance\">AggregateInstance</a>&lt;A&gt;","synthetic":false,"types":["chekov::aggregate::instance::AggregateInstance"]},{"text":"impl&lt;A:&nbsp;<a class=\"trait\" href=\"chekov/trait.Application.html\" title=\"trait chekov::Application\">Application</a>&gt; Actor for <a class=\"struct\" href=\"chekov/struct.SubscriberManager.html\" title=\"struct chekov::SubscriberManager\">SubscriberManager</a>&lt;A&gt;","synthetic":false,"types":["chekov::subscriber::manager::SubscriberManager"]}];
implementors["event_store"] = [{"text":"impl&lt;S&gt; Actor for <a class=\"struct\" href=\"event_store/struct.EventStore.html\" title=\"struct event_store::EventStore\">EventStore</a>&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: 'static + <a class=\"trait\" href=\"event_store/prelude/trait.Storage.html\" title=\"trait event_store::prelude::Storage\">Storage</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,&nbsp;</span>","synthetic":false,"types":["event_store::EventStore"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()